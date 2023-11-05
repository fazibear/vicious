use mos6510::memory::Memory;
use mos6510::CPU;
use mos6510::registers::Registers;
use mos6510::status_flags::StatusFlags;

use sid_file::SidFile;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use spinners::{Spinner, Spinners};

#[derive(Debug)]
pub struct PlayerMemory([u8; 65536]);

impl Memory for PlayerMemory {
    fn read(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        if (address & 0xfc00) == 0xd400 {
            println!("write to sid register {:04x} = {:02x}", address, value);
        } else {
            self.0[address as usize] = value;
        }
    }
}

impl Default for PlayerMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerMemory {
    pub fn new() -> Self {
        Self([0; 65536])
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        data.iter()
            .enumerate()
            .for_each(|(i, &b)| self.write((offset + i as u16), b));
    }
}

pub struct Player {
    playing: bool,
    sid_file: SidFile,
    cpu: CPU,
    current_song: u16,
    speed: Duration,
}

impl Player {
    pub fn new(data: &[u8]) -> Self {
        let sid_file = SidFile::parse(&data).expect("failed to read sid file");
        //println!("sid file: {:?}", sid_file);
        let current_song = sid_file.start_song;
        let sid_speed = if sid_file.speed == 0 { 50 } else { 100 };
        let speed = Duration::from_millis(1000 / sid_speed);
        let mut memory = PlayerMemory::new();
        println!("load address: {}", sid_file.real_load_address);
        memory.load(&sid_file.data, sid_file.real_load_address as u16);

        let cpu = CPU::new(Box::new(memory));

        Self {
            playing: false,
            sid_file,
            cpu,
            current_song,
            speed,
        }
    }

    pub fn init(&mut self) {
        if self.sid_file.play_address == 0 {
            self.jump_subroutine(self.sid_file.init_address as u16, 0);
            self.sid_file.play_address = ((self.cpu.get_memory_at(0x0315) as u16) << 8) + self.cpu.get_memory_at(0x0314) as u16;
            println!("new play_addr: {:04x}", self.sid_file.play_address);
        }
        println!("{}, {}", self.sid_file.songs, self.sid_file.start_song);
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
            self.jump_subroutine(
                self.sid_file.init_address as u16,
                (self.current_song - 1) as u8,
            );
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn step(&mut self) -> bool {
        if self.playing {
            self.cpu.step();
        }
        false
    }
    
    pub fn jump_subroutine(&mut self, program_counter: u16, accumulator: u8) -> u64 {
        let mut cycles = 0;
        
        println!("jump_subroutine: {}, {}", program_counter, accumulator);
        
        self.cpu.registers = Registers::new();
        self.cpu.status_flags = StatusFlags::new();
        
        self.cpu.registers.accumulator = accumulator;
        self.cpu.registers.program_counter = program_counter;

        self.cpu.push(0);
        self.cpu.push(0);
        
        while self.cpu.registers.program_counter > 1 {
            let stepc=  self.cpu.step();
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

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path).expect("failed to read file");

    let mut player = Player::new(&data);

    let mut sp = Spinner::new(Spinners::Dots, "Playing".into());
    
    player.init();
    
    loop {
        let now = Instant::now();
        if player.step() {
            break;
        }
        thread::sleep(player.speed - now.elapsed());
    }
    sp.stop();
}
