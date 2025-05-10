use anyhow::Result;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Sample, Stream, StreamConfig,
};

use rb::{RbConsumer, RbProducer, SpscRb, RB};

pub struct Sound {
    pub device: Device,
    pub config: StreamConfig,
    pub stream: Stream,
    pub buffer: SpscRb<i16>,
}

impl Sound {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let supported_config = device.default_output_config()?;
        let config = supported_config.into();

        let buffer = SpscRb::new(1024);

        let consumer = buffer.consumer();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut tmp: Vec<i16> = vec![0; data.len() / 2];
                    let _readed = consumer.read(&mut tmp[..]).unwrap_or(0);
                    //  let new = tmp.iter().map(|&s| f32::from_sample(s)).collect::<Vec<_>>();

                    let new = tmp
                        .iter()
                        .flat_map(|&s| {
                            let v = f32::from_sample(s);
                            vec![v, v]
                        })
                        .collect::<Vec<_>>();

                    data.copy_from_slice(&new);
                },
                move |err| {
                    dbg!("audio output error: {}", err);
                },
                None,
            )
            .expect("create a stream");

        Ok(Self {
            device,
            config,
            stream,
            buffer,
        })
    }

    pub fn info(&self) -> Result<()> {
        eprintln!("Output device: {}", self.device.name()?);
        eprintln!(
            "Supported stream config: {:?}",
            self.device.default_output_config()?
        );
        eprintln!("Stream config: {:?}", self.config);

        Ok(())
    }

    pub fn write_blocking(&mut self, data: &[i16]) {
        self.buffer.producer().write_blocking(data);
    }

    // pub fn stream(&self, dev_fn: impl FnMut(&mut [f32], &OutputCallbackInfo)) -> Result<(), Box<dyn std::error::Error>> {
    //     let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    //
    //     let stream = self.device
    //         .build_output_stream(&self.config, dev_fn, err_fn, None)?;
    //
    //     stream.play()?;
    //
    //     Ok(())
    // }
}
