use oxidrive_dsp::pedal::PedalNode;

pub enum WaveshaperParam {
    InputGain = 0,
    Drive     = 1,
    Asymmetric = 2,
    OutputLevel = 3,
}

impl From<WaveshaperParam> for usize {
    fn from(p: WaveshaperParam) -> usize { p as usize }
}

pub struct WaveshaperNode {
    bypass: bool,
    input_gain: f32,
    drive: f32,
    asymmetric: bool,
    output_level: f32,
}

impl WaveshaperNode {
    pub fn new() -> Self {
        Self {
            bypass: false,
            input_gain: 1.0,
            drive: 0.5,
            asymmetric: false,
            output_level: 1.0,
        }
    }
}

impl PedalNode for WaveshaperNode {
    fn prepare(&mut self, _sample_rate: u32, _max_buffer_size: usize) {}

    fn process(&mut self, data: &mut [f32]) {
        if self.bypass { return; }

        let total_gain = self.input_gain * (1.0 + self.drive * 15.0);
        let output_level = self.output_level;

        if self.asymmetric {
            for sample in data.iter_mut() {
                let driven = *sample * total_gain;
                
                // Determine if sample is negative (true = 1.0, false = 0.0)
                let is_neg = (driven < 0.0) as i32 as f32;
                
                // Calculate both paths without conditional branching
                let pos_val = driven.tanh();
                let neg_val = (driven * 1.5).tanh() * 0.8;
                
                // Blend results using the mask
                let shaped = pos_val + is_neg * (neg_val - pos_val);
                
                *sample = shaped * output_level;
            }
        } else {
            for sample in data.iter_mut() {
                let driven = *sample * total_gain;
                *sample = driven.tanh() * output_level;
            }
        }
    }

    fn name(&self) -> &str { "Waveshaper" }
    fn bypass(&self) -> bool { self.bypass }

    fn set_bypass(&mut self, bypass: bool) { self.bypass = bypass; }
    fn set_param(&mut self, param: usize, value: f32) {
        match param {
            0 => self.input_gain = value,
            1 => self.drive = value.clamp(0.0, 1.0),
            2 => self.asymmetric = value > 0.5,
            3 => self.output_level = value,
            _ => {}
        }
    }
}

