use anyhow::Result;
use mos6510rs::{Registers, StatusFlags, CPU};
use resid::{SamplingMethod, Sid};
use sid_file::SidFile;
use std::sync::{Arc, Mutex};

pub struct Player {
    pub cpu: CPU,
    pub sid: Arc<Mutex<Sid>>,
    pub playing: bool,
    pub sid_file: Option<SidFile>,
}

impl Player {
    pub fn new() -> Self {
        let sid = Arc::new(Mutex::new(Sid::new(resid::ChipModel::Mos8580)));
        sid.lock().expect("to unlock").set_sampling_parameters(
            SamplingMethod::Fast,
            985_248,
            48000,
        );

        sid.lock().expect("to lock").write(24, 15);

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

        Self {
            playing: false,
            sid,
            cpu,
            sid_file: None,
        }
    }

    pub fn load_data(&mut self, data: &[u8]) -> Result<()> {
        let sid_file = SidFile::parse(data)?;
        // let current_song = sid_file.start_song;
        // let speed_fraction = match sid_file.flags {
        //     Some(Flags {
        //         clock: Clock::NTSC, ..
        //     }) => 60,
        //     _ => 50,
        // };

        // let model = match sid_file.flags {
        //     Some(Flags {
        //         sid_model: sid_file::ChipModel::MOS6581,
        //         ..
        //     }) => resid::ChipModel::Mos6581,
        //     _ => resid::ChipModel::Mos8580,
        // };

        self.cpu
            .write_slice(&sid_file.data, sid_file.real_load_address);

        self.sid_file = Some(sid_file);

        Ok(())
    }

    pub fn sid_file(&self) -> &SidFile {
        self.sid_file.as_ref().expect("sid_file loaded")
    }

    pub fn change_track(&mut self, track: u16) {
        self.playing = true;

        if track > 0 && track <= self.sid_file().songs {
            self.stop();
            self.cpu.reset();
            self.jump_subroutine(self.sid_file().init_address, (track - 1) as u8);
        }
    }

    pub fn play(&mut self) {
        // if self.sid_file().play_address == 0 {
        //     dbg!("no play address");
        //     self.jump_subroutine(self.sid_file().init_address, 0);
        //     self.sid_file().play_address = self.cpu.read_word(0x0314);
        // }

        self.change_track(self.sid_file().start_song);
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn step(&mut self) {
        if self.playing && 0 == self.jump_subroutine(self.sid_file().play_address, 0) {
            self.playing = false;
        }
    }

    const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

    pub fn data(&mut self) -> Option<Vec<i16>> {
        println!("OK");

        if !self.playing {
            return None;
        }

        self.step();

        let mut delta: u32 = 20000;
        let mut buffer = vec![0; Self::BUFFER_SIZE];
        let mut samples_count = 0;
        while delta > 0 {
            let (samples, next_delta) =
                self.sid
                    .lock()
                    .expect("to lock")
                    .sample(delta, &mut buffer[samples_count..], 1);
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
