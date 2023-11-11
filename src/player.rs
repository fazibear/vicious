use mos6510::registers::Registers;
use mos6510::status_flags::StatusFlags;
use mos6510::CPU;

use std::time::Duration;

use sid_file::SidFile;

use resid::ChipModel;

use crate::sid::PlayerSid;
use crate::memory::PlayerMemory;


pub struct Player {
    pub playing: bool,
    pub sid_file: SidFile,
    pub cpu: CPU,
    pub current_song: u16,
    pub speed: Duration,
    pub samples_per_frame: u32,
}

impl Player {
    pub fn new(data: &[u8]) -> Self {
        let sid_file = SidFile::parse(data).expect("failed to read sid file");
        let current_song = sid_file.start_song;
        println!("sid file type: {:?}", sid_file.file_type);
        let speed = Duration::from_millis(1000 / 50);
        
        let chip = match sid_file.flags {
            Some(flags) => flags.sid_model,
            _ => sid_file::ChipModel::MOS8580,
        };
        
        let model = match chip { 
            sid_file::ChipModel::MOS6581 => ChipModel::Mos6581,
            _ => ChipModel::Mos8580,
        };

        let sid = PlayerSid::new(model);

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

    // pub fn play(&mut self) {
    //     self.playing = true;
    // }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn step(&mut self) -> bool {
        self.jump_subroutine(self.sid_file.play_address, 0) == 0
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

    // pub fn debug(&self) {
    //     println!(
    //         "{} {} {} {} {} {}",
    //         self.cpu.registers.x,
    //         self.cpu.registers.y,
    //         self.cpu.registers.accumulator,
    //         self.cpu.registers.stack_pointer,
    //         self.cpu.status_flags.to_byte(),
    //         self.cpu.registers.stack_pointer
    //     );
    // }
}


