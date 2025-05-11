use crate::memory::PlayerMemory;
use anyhow::Result;
use mos6510rs::registers::Registers;
use mos6510rs::status_flags::StatusFlags;
use mos6510rs::CPU;
use sid_file::{Clock, Flags, SidFile};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Player {
    pub playing: bool,
    pub sid_file: SidFile,
    pub cpu: CPU,
    pub memory: Rc<RefCell<PlayerMemory>>,
    pub current_song: u16,
    pub speed: Duration,
    // pub samples_per_frame: u32,
    pub last_step: Instant,
}

impl Player {
    pub fn new(data: &[u8]) -> Result<Self> {
        let sid_file = SidFile::parse(data)?;
        let current_song = sid_file.start_song;
        let speed_fraction = match sid_file.flags {
            Some(Flags {
                clock: Clock::NTSC, ..
            }) => 60,
            _ => 50,
        };

        let speed = Duration::from_millis(1000 / speed_fraction);

        let model = match sid_file.flags {
            Some(Flags {
                sid_model: sid_file::ChipModel::MOS6581,
                ..
            }) => resid::ChipModel::Mos6581,
            _ => resid::ChipModel::Mos8580,
        };

        let memory = Rc::new(RefCell::new(PlayerMemory::new(model, 48000)));

        memory
            .borrow_mut()
            .load(&sid_file.data, sid_file.real_load_address);

        memory.borrow_mut().sid.write(24, 15);

        let cpu = CPU::new(memory.clone());

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
            last_step: Instant::now(),
        })
    }

    pub fn change_track(&mut self, track: u16) {
        self.playing = true;
        if track > 0 && track <= self.sid_file.songs {
            self.stop();
            self.cpu.reset();
            self.current_song = track;
            self.jump_subroutine(self.sid_file.init_address, (self.current_song - 1) as u8);
        }
    }

    pub fn play(&mut self) {
        if self.sid_file.play_address == 0 {
            dbg!("no play address");
            self.jump_subroutine(self.sid_file.init_address, 0);
            self.sid_file.play_address = self.cpu.read_word(0x0314);
        }

        self.change_track(self.sid_file.start_song);
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn step(&mut self) {
        if self.playing && 0 == self.jump_subroutine(self.sid_file.play_address, 0) {
            self.playing = false;
        }
    }

    pub fn sid_file(&self) -> &SidFile {
        &self.sid_file
    }

    const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

    pub fn data(&mut self) -> Option<Vec<i16>> {
        if self.last_step.elapsed() < self.speed {
            return None;
        }

        if !self.playing {
            return None;
        }

        self.step();
        self.last_step = Instant::now();

        let mut delta: u32 = 20000;
        let mut buffer = vec![0; Self::BUFFER_SIZE];
        let mut samples_count = 0;
        while delta > 0 {
            let (samples, next_delta) =
                self.memory
                    .borrow_mut()
                    .sid_sample(delta, &mut buffer[samples_count..], 1);
            samples_count = samples;
            delta = next_delta;
        }
        buffer.resize(samples_count, 0);
        Some(buffer)
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
