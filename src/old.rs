mod memory;
mod player;
mod sound;

use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample,
};
use log::info;
use player::Player;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use sound::Sound;
use std::{thread, time::Instant};

const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let filename = std::env::args().nth(1).unwrap_or("".to_string());
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path)?;

    let buffer = HeapRb::<i16>::new(44100 * 2);
    let (mut prod, mut cons) = buffer.split();

    let sound = Sound::new()?;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let dev_rn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if data.len() / 2 > cons.occupied_len() {
            println!("ups!");
            return;
        }
        info!("{} samples received", data.len() / 2);
        let s: f32 = 15417.to_sample();
        info!("{} -> {}", 15417, s);
        for samples in data.chunks_mut(2) {
            let val = cons.try_pop().unwrap();
            let sample_val: f32 = val.to_sample();
            for sample in samples {
                *sample = sample_val;
            }
        }
    };

    let stream = sound
        .device
        .build_output_stream(&sound.config, dev_rn, err_fn, None)?;

    let _ = stream.play();

    let mut player = Player::new(&data)?;
    player.init();
    player.info();
    sound.info()?;

    let mut now: Instant;
    loop {
        now = Instant::now();

        if player.step() {
            dbg!("break");
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

            let wnow = Instant::now();
            prod.push_slice(&buffer[..samples]);
            info!("Write {} samples in {:?}.", samples, wnow.elapsed());
            delta = next_delta;
        }

        if let ((BUFFER_SIZE..), Some(time)) =
            (prod.occupied_len(), player.speed.checked_sub(now.elapsed()))
        {
            info!("Sleep {:?}", time);
            thread::sleep(time)
        }
    }

    Ok(())
}
