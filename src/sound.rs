use anyhow::Result;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, StreamConfig,
};

pub struct Sound {
    pub device: Device,
    pub config: StreamConfig,
}

impl Sound {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find output device");
        println!("Output device: {}", device.name()?);

        let supported_config = device.default_output_config().unwrap();
        println!("Supported stream config: {:?}", supported_config);

        let config = supported_config.into();
        println!("Stream config: {:?}", config);

        Ok(Self { device, config })
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
