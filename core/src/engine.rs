use anyhow::{Context, Result};
use cpal::{BufferSize, Device, DeviceId, Stream, StreamConfig, SupportedBufferSize, traits::{DeviceTrait, HostTrait}};
use rtrb::RingBuffer;

use crate::chain::{self, PedalChainHandle};

pub enum AudioDeviceType {
    Input,
    Output,
}

pub struct AudioDeviceInfo {
    pub name: String,
    pub id: DeviceId,
}

impl TryFrom<Device> for AudioDeviceInfo {
    type Error = anyhow::Error;

    fn try_from(device: Device) -> Result<Self> {
        Ok(AudioDeviceInfo {
            name: device.name().unwrap_or_else(|_| "Unknown Device".to_string()),
            id: device.id().context("failed to get device id")?,
        })
    }
}

pub trait AudioCallback: Send + 'static {
    fn process_frame(&mut self, data: &mut [f32]);
}

pub struct AudioEngine {
    pub input: Device,
    pub output: Device,

    input_stream: Option<Stream>,
    output_stream: Option<Stream>,

    callback: Option<Box<dyn AudioCallback>>,
}

impl AudioEngine {
    pub fn new(input: AudioDeviceInfo, output: AudioDeviceInfo) -> Result<Self> {
        log::info!("Using '{}' for input.", input.name);
        log::info!("Using '{}' for output.", output.name);

        let host = cpal::default_host();

        let input = host
            .device_by_id(&input.id)
            .context("failed to find input device")?;

        let output = host
            .device_by_id(&output.id)
            .context("failed to find output device")?;

        Ok(Self {
            input,
            output,
            input_stream: None,
            output_stream: None,
            callback: None,
        })
    }

    pub fn build_streams(&mut self) -> Result<()> {
        let config = self.input
            .default_input_config()
            .context("failed to get input config")?;

        let buffer_size = match config.buffer_size() {
            SupportedBufferSize::Range { min, max } => *min as usize, // use smallest supported
            SupportedBufferSize::Unknown => 1024,
        };

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: BufferSize::Fixed(buffer_size as u32),
        };

        let ring_size = buffer_size * 2;
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(ring_size);

        let input_stream = self.input
            .build_input_stream(
                stream_config,
                move |data: &[f32], _| {
                    if let Ok(chunk) = producer.write_chunk_uninit(data.len()) {
                        chunk.fill_from_iter(data.iter().copied());
                    }                
                },
                |err| log::error!("input stream error: {err}"),
                None,
            )
            .context("failed to build input stream")?;

        let output_stream = self.output
            .build_output_stream(
                stream_config,
                move |data: &mut [f32], _| {
                    if let Ok(chunk) = consumer.read_chunk(data.len()) {
                        let (first, second) = chunk.as_slices();
                        let mid = first.len();
                        data[..mid].copy_from_slice(first);
                        data[mid..].copy_from_slice(second);
                        chunk.commit_all();
                    } else {
                        data.fill(0.0);
                    }

                    if self.callback.is_some() {
                        self.callback.process_frame(data);
                    }
                },
                |err| log::error!("output stream error: {err}"),
                None,
            )
            .context("failed to build output stream")?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.input_stream.as_ref()
            .context("streams not built")?
            .play()
            .context("failed to start input")?;

        self.output_stream.as_ref()
            .context("streams not built")?
            .play()
            .context("failed to start output")?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.input_stream.as_ref()
            .context("streams not built")?
            .pause()
            .context("failed to pause input")?;

        self.output_stream.as_ref()
            .context("streams not built")?
            .pause()
            .context("failed to pause output")?;

        Ok(())
    }

    pub fn set_callback(&mut self, callback: impl AudioCallback) {
        self.callback = Some(Box::new(callback));
    }
}

pub fn list_devices(type_: AudioDeviceType) -> Result<Vec<AudioDeviceInfo>> {
    let host = cpal::default_host();

    let devices = match type_ {
        AudioDeviceType::Input => host
            .input_devices()
            .context("failed to get input devices")?,
        AudioDeviceType::Output => host
            .output_devices()
            .context("failed to get output devices")?,
    };

    devices
        .filter_map(|d| AudioDeviceInfo::try_from(d).ok())
        .collect::<Vec<_>>()
        .pipe(Ok)
}

