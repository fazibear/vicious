use mos6510rs::{Registers, StatusFlags, CPU};
use rb::{Producer, RbProducer};
use resid::{SamplingMethod, Sid};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub struct SidPlayer {
    cpu: CPU,
    sid: Arc<Mutex<Sid>>,
    init_address: u16,
    play_address: u16,
    songs: u16,
    current_song: u16,
    producer: Producer<i16>,
    playing: bool,
    last_step: Instant,
}

impl SidPlayer {
    pub fn new(producer: Producer<i16>, sample_rate: u32) -> Self {
        let mut sid = Sid::new(resid::ChipModel::Mos8580);
        sid.set_sampling_parameters(SamplingMethod::Fast, 985_248, sample_rate);
        sid.write(24, 15);

        let sid = Arc::new(Mutex::new(sid));

        let mut cpu = CPU::new();

        let cpu_sid = sid.clone();
        cpu.set_write_byte_callback(Box::new(move |address, value| {
            if (address & 0xfc00) == 0xd400 {
                cpu_sid
                    .lock()
                    .expect("to lock")
                    .write((address & 0x1f) as u8, value);
            }
        }));

        let last_step = Instant::now();

        Self {
            sid,
            cpu,
            producer,
            last_step,
            playing: false,
            init_address: 0,
            play_address: 0,
            songs: 0,
            current_song: 0,
        }
    }

    pub fn load_data(
        &mut self,
        data: &[u8],
        load_address: u16,
        init_addres: u16,
        play_address: u16,
        songs: u16,
        current_song: u16,
    ) {
        self.init_address = init_addres;
        self.play_address = play_address;
        self.songs = songs;
        self.current_song = current_song;

        self.cpu.write_slice(data, load_address);

        if self.play_address == 0 {
            println!("play == 0");
            self.jump_subroutine(self.init_address, 0);
            self.play_address = self.cpu.read_word(0x0314);
        }
    }

    const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

    pub fn wait_for_next_step(&mut self) -> bool {
        if self.last_step.elapsed() < Duration::from_millis(20) {
            return true;
        }

        if !self.playing {
            return true;
        }

        println!(
            "{:?}",
            (Instant::now() - self.last_step.elapsed()).elapsed()
        );
        self.last_step = Instant::now();
        false
    }

    pub fn step(&mut self) {
        if self.wait_for_next_step() {
            return;
        }

        if self.playing && 0 == self.jump_subroutine(self.play_address, 0) {
            println!("end?");
            self.playing = false;
        }

        let mut delta: u32 = 20000;
        let mut buffer = vec![0; Self::BUFFER_SIZE];
        let mut samples_count = 0;
        while delta > 0 {
            let (samples, next_delta) =
                self.sid
                    .lock()
                    .expect("to lock")
                    .sample(delta, &mut buffer[samples_count..], 1);
            samples_count += samples;
            delta = next_delta;
        }
        //TODO
        let _ = self.producer.write_blocking(&buffer[..samples_count]);
    }

    pub fn change_track(&mut self, track: u16) {
        self.playing = false;
        if track > 0 && track <= self.songs {
            self.cpu.reset();
            self.jump_subroutine(self.init_address, (track - 1) as u8);
            self.playing = true;
        }
    }

    pub fn play(&mut self) {
        self.change_track(self.current_song);
    }

    fn jump_subroutine(&mut self, program_counter: u16, accumulator: u8) -> u64 {
        let mut cycles = 0;

        self.cpu.registers = Registers::new();
        self.cpu.status_flags = StatusFlags::new();

        self.cpu.registers.accumulator = accumulator;
        self.cpu.registers.program_counter = program_counter;

        self.cpu.push(0);
        self.cpu.push(0);

        while self.cpu.registers.program_counter > 1 {
            let step_count = self.cpu.step();
            cycles += step_count;
        }
        cycles
    }
}
