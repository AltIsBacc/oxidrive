use std::io::Cursor;

use anyhow::Result;
use hound::{SampleFormat, WavReader};

pub fn load_ir(bytes: Vec<u8>) -> Result<Vec<f32>> {
    let cursor = Cursor::new(bytes);
    let mut reader = WavReader::new(cursor)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => {
            reader.samples::<f32>().map(|s| Ok(s?)).collect::<anyhow::Result<_>>()?
        }
        SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| Ok(s? as f32 / max)).collect::<anyhow::Result<_>>()?
        }
    };

    // IR wavs are often stereo — take only the left channel
    if spec.channels == 2 {
        Ok(samples.into_iter().step_by(2).collect())
    } else {
        Ok(samples)
    }
}

