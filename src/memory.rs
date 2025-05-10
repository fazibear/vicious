use resid::ChipModel;
use resid::SamplingMethod;
use resid::Sid;

use mos6510rs::memory::Memory;

pub struct PlayerMemory {
    pub memory: [u8; 65536],
    pub sid: Sid,
}

impl Memory for PlayerMemory {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        if (address & 0xfc00) == 0xd400 {
            //println!("lusid.poke({}, {})", address & 0x1f, value);
            self.sid.write((address & 0x1f) as u8, value);
        } else {
            self.memory[address as usize] = value;
        }
    }
}

impl PlayerMemory {
    pub fn new(model: ChipModel, sample_rate: u32) -> Self {
        let mut sid = Sid::new(model);
        sid.set_sampling_parameters(SamplingMethod::Fast, 985_248, sample_rate);
        Self {
            memory: [0; 65536],
            sid,
        }
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        for (i, &b) in data.iter().enumerate() {
            self.write(offset + i as u16, b);
        }
    }

    pub fn sid_sample(&mut self, delta: u32, buffer: &mut [i16], channels: usize) -> (usize, u32) {
        self.sid.sample(delta, buffer, channels)
    }
}
