use oxidrive_dsp::{cpal::Sample, engine::{buffer::AudioBuffer, streams::ResolvedStreamConfig}, pedal::{NodeControls, NodeControlsBase, PedalNode}};

pub struct SilenceNode {
    controls: NodeControlsBase,
}

impl SilenceNode {
    pub fn new() -> Self {
        Self {
            controls: NodeControlsBase::default(),
        }
    }
}

impl NodeControls for SilenceNode {
    #[inline]
    fn controls(&self) ->  &NodeControlsBase { &self.controls }

    #[inline]
    fn controls_mut(&mut self) ->  &mut NodeControlsBase { &mut self.controls }
}

impl PedalNode for SilenceNode {
    fn prepare(&mut self, _config: &ResolvedStreamConfig) { }

    fn process(&mut self, data: &mut AudioBuffer<'_, f32>) {
        data.interleaved().fill(f32::EQUILIBRIUM);
    }

    fn name(&self) -> &str { "Silence" }
    fn set_param_raw(&mut self, _param: u32, _value: f32) { }
}

