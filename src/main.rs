use anyhow::Result;
use ringbuf::HeapRb;
use std::{thread, time::Instant};

mod memory;
mod player;
mod sound;

use player::Player;
use sound::Sound;

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample,
};

const BUFFER_SIZE: usize = 2i32.pow(13) as usize;

fn main() -> Result<()> {
    let filename = std::env::args().nth(1).unwrap_or("".to_string());
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path)?;

    let buffer = HeapRb::<i16>::new(44100 * 2);
    let (mut prod, mut cons) = buffer.split();

    let sound = Sound::new()?;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let dev_rn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if data.len() / 2 > cons.len() {
            println!("ups!");
            return;
        }
        for samples in data.chunks_mut(2) {
            let val = cons.pop().unwrap();
            let sample_val = i16::from_sample(val).to_sample::<f32>();
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
            prod.push_slice(&buffer[..samples]);
            delta = next_delta;
        }

        if let ((BUFFER_SIZE..), Some(time)) = (prod.len(), player.speed.checked_sub(now.elapsed()))
        {
            thread::sleep(time)
        }
    }

    Ok(())
}
