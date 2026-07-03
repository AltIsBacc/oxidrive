use anyhow::{Context, Result};
use cpal::{Device, SizedSample, traits::{DeviceTrait, StreamTrait}};
use rtrb::RingBuffer;

use crate::engine::{buffer::AudioBuffer, streams::{AudioStream, ResolvedStreamConfig}};

pub mod streams;
pub mod buffer;

pub struct AudioEngine {
    pub input: Device,
    pub output: Device,

    input_stream: Option<AudioStream>,
    output_stream: Option<AudioStream>,
}

impl AudioEngine {
    pub fn new(input: Device, output: Device) -> Result<Self> {
        Ok(Self {
            input,
            output,
            input_stream: None,
            output_stream: None,
        })
    }

    pub fn build_streams<T>(
        &mut self, mut callback: impl AudioCallback<T>
    ) -> Result<()>
    where
        T: SizedSample + Send + 'static
    {
        let input_config: ResolvedStreamConfig = self.input.default_input_config()?.into();
        
        callback.prepare(input_config.sample_rate, input_config.buffer_size as usize);

        let (producer, consumer) = RingBuffer::<T>::new((input_config.buffer_size * 2) as usize);
        let input = AudioStream::new_input::<T>(
            &self.input, input_config, producer
        )?;

        let output_config: ResolvedStreamConfig = self.output.default_output_config()?.into();
        let output = AudioStream::new_output::<T>(
            &self.output, output_config,
            consumer, callback
        )?;

        self.input_stream = Some(input);
        self.output_stream = Some(output);

        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.input_stream
            .as_ref()
            .context("streams not built")?
            .stream
            .play()
            .context("failed to start input")?;

        self.output_stream
            .as_ref()
            .context("streams not built")?
            .stream
            .play()
            .context("failed to start output")?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.input_stream
            .as_ref()
            .context("streams not built")?
            .stream
            .pause()
            .context("failed to pause input")?;

        self.output_stream
            .as_ref()
            .context("streams not built")?
            .stream
            .pause()
            .context("failed to pause output")?;

        Ok(())
    }
}

pub trait AudioCallback<T>: Send + 'static
where
    T: SizedSample
{
    fn prepare(&mut self, sample_rate: u32, buffer_size: usize);
    fn process_frame(&mut self, data: &mut AudioBuffer<'_, T>);
}

