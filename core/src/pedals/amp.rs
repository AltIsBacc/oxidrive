use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Q_BUTTERWORTH_F32, Type};
use oxidrive_dsp::pedal::PedalNode;

use crate::pedals::{cabinet::CabinetNode, waveshaper::WaveshaperNode};

pub enum AmpParam {
    InputGain      = 0,
    Drive          = 1,
    Asymmetric     = 2,
    Bass           = 3,
    Mid            = 4,
    Treble         = 5,
    OutputLevel    = 6,
    CabinetEnabled = 7,
}

impl From<AmpParam> for usize {
    fn from(p: AmpParam) -> usize { p as usize }
}

pub struct AmpNode {
    bypass: bool,
    sample_rate: u32,

    waveshaper: WaveshaperNode,

    bass: DirectForm1<f32>,
    mid: DirectForm1<f32>,
    treble: DirectForm1<f32>,
    bass_gain: f32,
    mid_gain: f32,
    treble_gain: f32,

    output_level: f32,

    cabinet: CabinetNode,
}

impl AmpNode {
    pub fn new(ir: Vec<f32>) -> Self {
        let dummy_sr = 44100.hz();
        let flat = Coefficients::<f32>::from_params(
            Type::PeakingEQ(0.0), dummy_sr, 1000.0.hz(), Q_BUTTERWORTH_F32
        ).unwrap();

        Self {
            bypass: false,
            sample_rate: 44100,
            waveshaper: WaveshaperNode::new(),
            bass:   DirectForm1::new(flat),
            mid:    DirectForm1::new(flat),
            treble: DirectForm1::new(flat),
            bass_gain: 0.0,
            mid_gain: 0.0,
            treble_gain: 0.0,
            output_level: 1.0,
            cabinet: CabinetNode::new(ir),
        }
    }

    pub fn set_ir(&mut self, ir: Vec<f32>) {
        self.cabinet.set_ir(ir);
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate.hz();

        self.bass = DirectForm1::new(
            Coefficients::<f32>::from_params(
                Type::LowShelf(self.bass_gain), sr, 80.0.hz(), Q_BUTTERWORTH_F32
            ).unwrap()
        );
        self.mid = DirectForm1::new(
            Coefficients::<f32>::from_params(
                Type::PeakingEQ(self.mid_gain), sr, 800.0.hz(), Q_BUTTERWORTH_F32
            ).unwrap()
        );
        self.treble = DirectForm1::new(
            Coefficients::<f32>::from_params(
                Type::HighShelf(self.treble_gain), sr, 4000.0.hz(), Q_BUTTERWORTH_F32
            ).unwrap()
        );
    }
}

impl PedalNode for AmpNode {
    fn prepare(&mut self, sample_rate: u32, max_buffer_size: usize) {
        self.sample_rate = sample_rate;
        self.waveshaper.prepare(sample_rate, max_buffer_size);
        self.cabinet.prepare(sample_rate, max_buffer_size);
        self.update_filters();
    }

    fn process(&mut self, data: &mut [f32]) {
        if self.bypass { return; }

        self.waveshaper.process(data);

        for sample in data.iter_mut() {
            *sample = self.bass.run(*sample);
            *sample = self.mid.run(*sample);
            *sample = self.treble.run(*sample);
            *sample *= self.output_level;
        }

        self.cabinet.process(data);
    }

    fn name(&self) -> &str { "Amp" }
    fn bypass(&self) -> bool { self.bypass }
    fn set_bypass(&mut self, bypass: bool) { self.bypass = bypass; }

    fn set_param(&mut self, param: usize, value: f32) {
        match param {
            0 => self.waveshaper.set_param(0, value), // InputGain
            1 => self.waveshaper.set_param(1, value), // Drive
            2 => self.waveshaper.set_param(2, value), // Asymmetric
            3 => { self.bass_gain = value;   self.update_filters(); }
            4 => { self.mid_gain = value;    self.update_filters(); }
            5 => { self.treble_gain = value; self.update_filters(); }
            6 => self.output_level = value,
            7 => self.cabinet.set_bypass(value > 0.5),    // CabinetEnabled
            _ => {}
        }
    }
}

