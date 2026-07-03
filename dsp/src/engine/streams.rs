use anyhow::Result;
use cpal::{BufferSize, Device, SizedSample, Stream, StreamConfig, SupportedBufferSize, SupportedStreamConfig, traits::DeviceTrait};
use rtrb::{Consumer, Producer};

use crate::engine::{AudioCallback, buffer::AudioBuffer};

#[derive(Clone, Copy)]
pub struct ResolvedStreamConfig {
    pub buffer_size: u32,
    pub channels: u16,
    pub sample_rate: u32,
}

impl From<SupportedStreamConfig> for ResolvedStreamConfig {
    fn from(value: SupportedStreamConfig) -> Self {
        let buffer_size = match value.buffer_size() {
            SupportedBufferSize::Range { min, max: _ } => *min,
            SupportedBufferSize::Unknown => 1024,
        };

        Self {
            buffer_size,
            channels: value.channels(),
            sample_rate: value.sample_rate(),
        }
    }
}

impl Into<StreamConfig> for ResolvedStreamConfig {
    fn into(self) -> StreamConfig {
        StreamConfig {
            buffer_size: BufferSize::Fixed(self.buffer_size),
            channels: self.channels,
            sample_rate: self.sample_rate,
        }
    }
}

pub struct AudioStream {
    pub config: ResolvedStreamConfig,
    pub stream: Stream,
}   

impl AudioStream {
    pub fn new_input<T>(
        device: &Device,
        config: ResolvedStreamConfig,
        mut dst: Producer<T>
    ) -> Result<Self>
    where 
        T: SizedSample + Send + 'static,
    {
        let stream = device.build_input_stream(
            config.into(),
            move |d, _| {
                if dst.push_entire_slice(d).is_err() {
                    log::warn!("input ring buffer full! drop frame detected.");
                }
            },
            |err| log::error!("error in input stream! {err}"),
            None
        )?;

        Ok(Self { config, stream })
    }

    pub fn new_output<T>(
        device: &Device,
        config: ResolvedStreamConfig,
        mut src: Consumer<T>,
        mut callback: impl AudioCallback<T>,
    ) -> Result<Self>
    where
        T: SizedSample + Send + 'static,
    {
        let stream = device.build_output_stream(
            config.into(),
            move |d, _| {
                if src.pop_entire_slice(d).is_err() {
                    d.fill(T::EQUILIBRIUM);
                }

                let mut wrapped = AudioBuffer::wrap(d, config.channels);
                callback.process_frame(&mut wrapped);
            },
            |err| log::error!("error in output stream! {err}"),
            None
        )?;
           
        Ok(Self { config, stream })
    }

}

