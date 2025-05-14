use anyhow::{Context, Result};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Sample, Stream, StreamConfig,
};

use log::info;
use rb::{RbConsumer, RbInspector, RbProducer, SpscRb, RB};

pub struct Playback {
    pub device: Device,
    pub config: StreamConfig,
    pub stream: Stream,
    pub buffer: SpscRb<i16>,
}

impl Playback {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("default output device available")?;
        let supported_config = device.default_output_config()?;
        let config = supported_config.into();

        let buffer = SpscRb::new(44100 * 2);

        let consumer = buffer.consumer();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                info!("sound buffer len: {}", data.len());
                let mut tmp: Vec<i16> = vec![0; data.len() / 2];
                let readed = consumer.read(&mut tmp[..]).unwrap_or(0);
                info!("{} samples received", readed == data.len() / 2);
                if readed < data.len() / 2 {
                    return;
                }
                let new = tmp
                    .iter()
                    .flat_map(|&s| {
                        let v = s.to_sample();
                        vec![v, v]
                    })
                    .collect::<Vec<_>>();

                data.copy_from_slice(&new[..]);

                // let f: f32 = 15417.to_sample();
                // info!("{} -> {}", 15417, f);
                // for samples in data.chunks_mut(2) {
                //     let val = tmp.pop().unwrap();
                //     let sample_val: f32 = val.to_sample();
                //     for sample in samples {
                //         *sample = sample_val;
                //     }
                // }
            },
            move |err| {
                dbg!("audio output error: {}", err);
            },
            None,
        )?;

        Ok(Self {
            device,
            config,
            stream,
            buffer,
        })
    }

    pub fn write_blocking(&mut self, data: &[i16]) {
        self.buffer.producer().write_blocking(data);
    }

    pub fn count(&self) -> usize {
        self.buffer.count()
    }
}
