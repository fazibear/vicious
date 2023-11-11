use std::thread;
use std::time::Instant;

use ringbuf::HeapRb;

mod memory;
mod player;
mod sid;
mod sound;

use player::Player;
use sound::Sound;

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample,
};

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path).expect("failed to read file");

    let buffer = HeapRb::<i16>::new(44100 * 2);
    let (mut prod, mut cons) = buffer.split();

    let sound = Sound::new().expect("failed to initialize sound");

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let dev_rn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if data.len() / 2 > cons.len() {
            println!("ups!");
            return;
        }
        for samples in data.chunks_mut(2) {
            let val = cons.pop().expect("za mao danych");
            let sample_val = i16::from_sample(val).to_sample::<f32>();
            for sample in samples {
                *sample = sample_val;
            }
        }
    };

    let stream = sound
        .device
        .build_output_stream(&sound.config, dev_rn, err_fn, None)
        .unwrap();

    let _ = stream.play();

    let mut player = Player::new(&data);
    player.init();

    println!("samples per frame: {}", player.samples_per_frame);
    loop {
        let now = Instant::now();

        if player.step() {
            break;
        }

        let mut delta: u32 = player.samples_per_frame * 100 / 2;
        while delta > 0 {
            let mut buffer = [0i16; 441];
            let (samples, next_delta) = player.cpu.sid.as_mut().samples(delta, &mut buffer[..]);
            prod.push_slice(&buffer[..samples]);
            delta = next_delta;
        }
        if prod.len() > 44100 / 2 {
            thread::sleep(player.speed - now.elapsed());
        }
    }
}
