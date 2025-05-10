mod memory;
mod player;
mod sound;

use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample,
};
use player::Player;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use sound::Sound;
use std::{thread, time::Instant};

const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

fn main() -> Result<()> {
    let filename = std::env::args().nth(1).unwrap_or("".to_string());
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path)?;

    let mut sound = Sound::new()?;
    let _ = sound.stream.play();

    let mut player = Player::new(&data)?;
    player.init();
    player.info();
    sound.info()?;

    let mut now: Instant;
    loop {
        now = Instant::now();

        if player.step() {
            break;
        }

        let mut delta: u32 = 22050;
        while delta > 0 {
            let mut buffer = [0i16; BUFFER_SIZE];
            let (samples, next_delta) =
                player
                    .memory
                    .borrow_mut()
                    .sid_sample(delta, &mut buffer[..], 1);
            sound.write_blocking(&buffer[..samples]);
            delta = next_delta;
        }
    }

    Ok(())
}
