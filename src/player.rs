use crate::memory::PlayerMemory;
use anyhow::Result;
use inline_colorization::*;
use mos6510::registers::Registers;
use mos6510::status_flags::StatusFlags;
use mos6510::CPU;
use resid::ChipModel;
use sid_file::SidFile;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct Player {
    pub playing: bool,
    pub sid_file: SidFile,
    pub cpu: CPU,
    pub memory: Rc<RefCell<PlayerMemory>>,
    pub current_song: u16,
    pub speed: Duration,
    // pub samples_per_frame: u32,
}

impl Player {
    pub fn new(data: &[u8]) -> Result<Self> {
        let sid_file = SidFile::parse(data)?;
        let current_song = sid_file.start_song;
        let speed = Duration::from_millis(1000 / 50);

        let chip = match sid_file.flags {
            Some(flags) => flags.sid_model,
            _ => sid_file::ChipModel::MOS8580,
        };

        let model = match chip {
            sid_file::ChipModel::MOS6581 => ChipModel::Mos6581,
            _ => ChipModel::Mos8580,
        };

        let memory = Rc::new(RefCell::new(PlayerMemory::new(model)));

        memory
            .borrow_mut()
            .load(&sid_file.data, sid_file.real_load_address);

        let cpu = CPU::new(memory.clone(), false);

        // let mut refresh_cia = (20000.0 * cpu.read_word(0xdc04) as f32 / 0x4c00 as f32).floor() as u16;
        // if refresh_cia == 0 || sid_file.speed == 0 {
        //     refresh_cia = 20000;
        // }
        // let samples_per_frame = (44100.0 / 2.0 * refresh_cia as f64 / 1000000.0).floor() as u32;

        Ok(Self {
            playing: false,
            sid_file,
            cpu,
            current_song,
            speed,
            memory: memory.clone(),
        })
    }

    pub fn init(&mut self) {
        if self.sid_file.play_address == 0 {
            self.jump_subroutine(self.sid_file.init_address, 0);
            self.sid_file.play_address = self.cpu.read_word(0x0314);
        }

        self.memory.borrow_mut().sid.write(24, 15);

        self.change_track(self.sid_file.start_song);
    }

    pub fn info(&self) {
        println!("------------------------------------");
        println!(
            "{color_yellow}Song:     {color_blue}{}{color_reset}",
            self.sid_file.name
        );
        println!(
            "{color_yellow}Author:   {color_blue}{}{color_reset}",
            self.sid_file.author
        );
        println!(
            "{color_yellow}Released: {color_blue}{}{color_reset}",
            self.sid_file.released
        );
        println!(
            "{color_yellow}Songs:    {color_blue}{}{color_reset}",
            self.sid_file.songs
        );
        println!("------------------------------------");
        println!(
            "{color_cyan}Data length:  {color_green}{}{color_reset}",
            self.sid_file.data.len()
        );
        println!(
            "{color_cyan}Init address: {color_green}0x{:04x}{color_reset}",
            self.sid_file.init_address
        );
        println!(
            "{color_cyan}Play address: {color_green}0x{:04x}{color_reset}",
            self.sid_file.play_address
        );
        println!(
            "{color_cyan}Load address: {color_green}0x{:04x}{color_reset}",
            self.sid_file.load_address
        );
        println!(
            "{color_cyan}Real load address: {color_green}0x{:04x}{color_reset}",
            self.sid_file.real_load_address
        );
        println!("------------------------------------");
        if let Some(flags) = self.sid_file.flags {
            println!("Clock speed: {:?}", flags.clock);
            println!("SID model 1: {:?}", flags.sid_model);
            println!("SID model 2: {:?}", flags.sid_model);
            println!("SID model 3: {:?}", flags.sid_model);
            println!("------------------------------------");
        }
    }

    pub fn change_track(&mut self, track: u16) {
        if track > 0 && track <= self.sid_file.songs {
            self.stop();
            self.cpu.reset();
            self.current_song = track;
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
        0 == self.jump_subroutine(self.sid_file.play_address, 0)
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
            let step_count = self.cpu.step();
            cycles += step_count;
        }
        cycles
    }
}
