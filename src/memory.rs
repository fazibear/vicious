use mos6510::memory::Memory;

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


