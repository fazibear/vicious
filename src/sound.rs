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
        let device = host.default_output_device().expect("no output device available");
        let supported_config = device.default_output_config()?;
        let config = supported_config.into();
        
        Ok(Self { device, config })
    }
    
    pub fn info(&self) -> Result<()> {
        eprintln!("Output device: {}", self.device.name()?);
        eprintln!("Supported stream config: {:?}", self.device.default_output_config()?);
        eprintln!("Stream config: {:?}", self.config);

        Ok(())
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
