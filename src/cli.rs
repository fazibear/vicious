mod app;
mod memory;
mod player;
mod sound;

use anyhow::Result;
use cpal::traits::StreamTrait;
use log::*;
use player::Player;
use sound::Sound;
use std::{thread, time::Instant};

const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

fn main() -> Result<()> {
    pretty_env_logger::init();
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

        let mut delta: u32 = 20000;
        while delta > 0 {
            let mut buffer = [0i16; BUFFER_SIZE];
            let (samples, next_delta) =
                player
                    .memory
                    .borrow_mut()
                    .sid_sample(delta, &mut buffer[..], 1);
            let wnow = Instant::now();
            sound.write_blocking(&buffer[..samples]);
            //println!("in {:?}", &buffer[..10]);
            info!("Write {} samples in {:?}.", samples, wnow.elapsed());
            delta = next_delta;
        }

        if let ((BUFFER_SIZE..), Some(time)) =
            (sound.count(), player.speed.checked_sub(now.elapsed()))
        {
            info!("Sleep {:?}", time);
            thread::sleep(time)
        }
    }

    Ok(())
}
