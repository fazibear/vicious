use mos6510::sid::Sid as SidT;
use resid::ChipModel;
use resid::Sid;

pub struct PlayerSid {
    pub sid: Sid,
}

impl SidT for PlayerSid {
    fn samples(&mut self, delta: u32, buffer: &mut [i16]) -> (usize, u32) {
        self.sid.sample(delta, buffer, 1)
    }

    fn write(&mut self, address: u8, value: u8) {
        self.sid.write(address, value);
    }
}

impl PlayerSid {
    pub fn new(model: ChipModel) -> Self {
        let sid = Sid::new(model);
        Self { sid }
    }
}


