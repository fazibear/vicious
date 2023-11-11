use anyhow::Result;

use mos6510::memory::Memory;
use mos6510::registers::Registers;
use mos6510::sid::Sid as SidT;
use mos6510::status_flags::StatusFlags;
use mos6510::CPU;

use resid::ChipModel::{Mos6581, Mos8580};
use resid::Sid;

use sid_file::SidFile;
use spinners::{Spinner, Spinners};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use ringbuf::HeapRb;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Sample, StreamConfig,
};

pub struct PlayerSid {
    pub sid: Sid,
}

impl SidT for PlayerSid {
    fn samples(&mut self, delta: u32, buffer: &mut [i16]) -> (usize, u32) {
        self.sid.sample(delta, buffer, 1)
    }

    fn write(&mut self, address: u8, value: u8) {
        //println!("write to sid register {} = {}", address, value);
        self.sid.write(address, value);
    }
}

impl PlayerSid {
    pub fn new() -> Self {
        let sid = Sid::new(Mos8580);
        Self { sid }
    }
}

pub struct PlayerMemory {
    pub memory: [u8; 65536],
}

impl Memory for PlayerMemory {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}

impl PlayerMemory {
    pub fn new() -> Self {
        Self { memory: [0; 65536] }
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        for (i, &b) in data.iter().enumerate() {
            self.write(offset + i as u16, b);
        }
    }
}

pub struct Player {
    playing: bool,
    sid_file: SidFile,
    cpu: CPU,
    current_song: u16,
    speed: Duration,
    samples_per_frame: u32,
}

impl Player {
    pub fn new(data: &[u8]) -> Self {
        let sid_file = SidFile::parse(data).expect("failed to read sid file");
        //println!("sid file: {:?}", sid_file);
        let current_song = sid_file.start_song;
        println!("sid file type: {:?}", sid_file.file_type);
        let speed = Duration::from_millis(1000 / 50);

        let sid = PlayerSid::new();

        let mut memory = PlayerMemory::new();
        println!("load address: {}", sid_file.real_load_address);

        memory.load(&sid_file.data, sid_file.real_load_address);

        let cpu = CPU::new(Box::new(memory), Box::new(sid));

        let mut refresh_cia = (20000.0
            * (cpu.get_memory_at(0xdc04) as u16 | ((cpu.get_memory_at(0xdc05) as u16) << 8)) as f32
            / 0x4c00 as f32)
            .floor() as u16;

        if refresh_cia == 0 || sid_file.speed == 0 {
            refresh_cia = 20000;
        }

        let samples_per_frame = (44100.0 / 2.0 * refresh_cia as f64 / 1000000.0).floor() as u32;

        Self {
            playing: false,
            sid_file,
            cpu,
            current_song,
            speed,
            samples_per_frame,
        }
    }

    pub fn init(&mut self) {
        if self.sid_file.play_address == 0 {
            self.jump_subroutine(self.sid_file.init_address, 0);
            self.sid_file.play_address = ((self.cpu.get_memory_at(0x0315) as u16) << 8)
                + self.cpu.get_memory_at(0x0314) as u16;
            println!("new play_addr: {:04x}", self.sid_file.play_address);
        }
        println!("{}, {}", self.sid_file.songs, self.sid_file.start_song);
        self.cpu.sid.write(24, 15); // turn up volume

        self.change_track(self.sid_file.start_song);
    }

    pub fn change_track(&mut self, track: u16) {
        if track > 0 && track <= self.sid_file.songs {
            println!("changing track to {}", track - 1);
            self.stop();
            self.cpu.reset();
            self.current_song = track;
            println!("init address: {}", self.sid_file.init_address);
            println!("current song: {}", self.current_song - 1);
            self.jump_subroutine(self.sid_file.init_address, (self.current_song - 1) as u8);
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn step(&mut self) -> bool {
        self.jump_subroutine(self.sid_file.play_address, 0);

        false
    }

    pub fn jump_subroutine(&mut self, program_counter: u16, accumulator: u8) -> u64 {
        let mut cycles = 0;

        self.cpu.registers = Registers::new();
        self.cpu.status_flags = StatusFlags::new();

        self.cpu.registers.accumulator = accumulator;
        self.cpu.registers.program_counter = program_counter;

        self.cpu.push(0);
        self.cpu.push(0);

        while self.cpu.registers.program_counter > 1 {
            //println!("{}", self.cpu.registers.program_counter);
            let stepc = self.cpu.step();
            //println!("Cycles: {}", stepc);
            cycles += stepc;
        }
        //println!("------------------------------------");
        cycles
    }

    pub fn debug(&self) {
        println!(
            "{} {} {} {} {} {}",
            self.cpu.registers.x,
            self.cpu.registers.y,
            self.cpu.registers.accumulator,
            self.cpu.registers.stack_pointer,
            self.cpu.status_flags.to_byte(),
            self.cpu.registers.stack_pointer
        );
    }
}

fn setup_sound() -> Result<(Device, StreamConfig)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find output device");
    println!("Output device: {}", device.name()?);

    let supported_config = device.default_output_config().unwrap();
    println!("Supported stream config: {:?}", supported_config);

    let config = supported_config.into();
    println!("Stream config: {:?}", config);

    Ok((device, config))
}

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path).expect("failed to read file");

    let buffer = HeapRb::<i16>::new(44100 * 2);
    let (mut prod, mut cons) = buffer.split();

    let (device, config) = setup_sound().expect("failed to setup sound");

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let dev_rn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if data.len() / 2 > cons.len() {
            println!("ups!");
            return;
        }
        for samples in data.chunks_mut(2) {
            let val = cons.pop().expect("za mao danych");
            let sample_val = i16::from_sample(val).to_sample::<f32>();
            for sample in samples {
                *sample = sample_val;
            }
        }
    };

    let stream = device
        .build_output_stream(&config, dev_rn, err_fn, None)
        .unwrap();

    let _ = stream.play();

    let mut player = Player::new(&data);
    player.init();

    println!("samples per frame: {}", player.samples_per_frame);
    loop {
        let now = Instant::now();

        if player.step() {
            break;
        }

        let mut delta: u32 = 441*100/2; // TODO:why?
        while delta > 0 {
            let mut buffer = [0i16; 441];
            let (samples, next_delta) = player.cpu.sid.as_mut().samples(delta, &mut buffer[..]);
            println!("samples: {}", samples);
            for sample in 0..samples {
                prod.push(buffer[sample]).expect("za duzo danych");
            }
            delta = next_delta;
        }
        println!("---");
        if prod.len() > 44100 / 2 {
            thread::sleep(player.speed - now.elapsed());
        }
    }
}
