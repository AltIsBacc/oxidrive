use num_enum::{IntoPrimitive, TryFromPrimitive};
use oxidrive_dsp::{engine::{buffer::AudioBuffer, streams::ResolvedStreamConfig}, pedal::{NodeControls, NodeControlsBase, PedalNode}, traits::normalized::Normalized};

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum WaveshaperParam {
    Drive = 0,
    Asymmetric = 1,
}

pub struct WaveshaperParams {
    pub drive: f32,
    pub asymmetric: bool,
}

impl WaveshaperParams {
    pub fn new() -> Self {
        Self {
            drive: 0.5,
            asymmetric: false,
        }
    }
}

pub struct WaveshaperNode {
    controls: NodeControlsBase,
    params: WaveshaperParams,
}

impl WaveshaperNode {
    pub fn new() -> Self {
        Self {
            controls: NodeControlsBase::default(),
            params: WaveshaperParams::new(),
        }
    }
}

impl NodeControls for WaveshaperNode {
    fn controls(&self) ->  &NodeControlsBase { &self.controls }
    fn controls_mut(&mut self) ->  &mut NodeControlsBase { &mut self.controls }
}

impl PedalNode for WaveshaperNode {
    fn prepare(&mut self, _config: &ResolvedStreamConfig) { }

    fn process(&mut self, data: &mut AudioBuffer<'_, f32>) {
        if self.params.drive == 0.0 { return; }

        let interleaved_data = data.interleaved();

        let output_gain = self.controls.output_gain();
        let k = self.params.drive * 15.0;
    
        if self.params.asymmetric {
            for sample in interleaved_data.iter_mut() {
                let driven = *sample;
                let is_neg = (driven < 0.0) as i32 as f32;
                
                // Positive cycle curve
                let pos_val = (driven * (1.0 + k)) / (1.0 + k * driven.abs());
                // Negative cycle curve (driven harder by multiplying k by 1.5)
                let k_neg = k * 1.5;
                let neg_val = ((driven * (1.0 + k_neg)) / (1.0 + k_neg * driven.abs())) * 0.8;
                
                let shaped = pos_val + is_neg * (neg_val - pos_val);
                *sample = shaped * output_gain;
            }
        } else {
            for sample in interleaved_data.iter_mut() {
                let driven = *sample;
                *sample = ((driven * (1.0 + k)) / (1.0 + k * driven.abs())) * output_gain;
            }
        }
    }

    fn name(&self) -> &str { "Waveshaper" }
    fn set_param_raw(&mut self, param: u32, value: f32) {
        if let Ok(casted) = param.try_into() {
            match casted {
                WaveshaperParam::Drive => self.params.drive = value.clamp(0.0, 10.0),
                WaveshaperParam::Asymmetric => self.params.asymmetric = value.to_bool(),
            }
        }
    }
}

