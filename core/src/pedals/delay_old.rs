use oxidrive_dsp::pedal::PedalNode;

pub enum DelaySubdivision {
    Whole           = 0,
    Half            = 1,
    Quarter         = 2,
    Eighth          = 3,
    Sixteenth       = 4,
    HalfTriplet     = 5,
    QuarterTriplet  = 6,
    EighthTriplet   = 7,
}

impl DelaySubdivision {
    pub fn multiplier(&self) -> f32 {
        match self {
            Self::Whole          => 4.0,
            Self::Half           => 2.0,
            Self::Quarter        => 1.0,
            Self::Eighth         => 0.5,
            Self::Sixteenth      => 0.25,
            Self::HalfTriplet    => 2.0 / 3.0,
            Self::QuarterTriplet => 1.0 / 1.5,
            Self::EighthTriplet  => 1.0 / 3.0,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Whole,
            1 => Self::Half,
            2 => Self::Quarter,
            3 => Self::Eighth,
            4 => Self::Sixteenth,
            5 => Self::HalfTriplet,
            6 => Self::QuarterTriplet,
            _ => Self::EighthTriplet,
        }
    }
}

impl From<DelaySubdivision> for f32 {
    fn from(s: DelaySubdivision) -> f32 { s.multiplier() }
}

pub enum DelayParam {
    DelayTime   = 0,
    Feedback    = 1,
    Mix         = 2,
    BpmSync     = 3,
    Bpm         = 4,
    Subdivision = 5,
    PingPong    = 6,
}

impl From<DelayParam> for usize {
    fn from(p: DelayParam) -> usize { p as usize }
}

pub struct DelayNode {
    bypass: bool,
    sample_rate: f32,

    delay_time: f32,
    feedback: f32,
    mix: f32,

    bpm_sync: bool,
    bpm: f32,
    subdivision: usize,

    ping_pong: bool,
    ping: bool,

    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_head: usize,
}

const MAX_DELAY_SECS: f32 = 5.0;

impl DelayNode {
    pub fn new() -> Self {
        Self {
            bypass: false,
            sample_rate: 44100.0,
            delay_time: 0.3,
            feedback: 0.4,
            mix: 0.5,
            bpm_sync: false,
            bpm: 120.0,
            subdivision: 2,
            ping_pong: false,
            ping: false,
            buffer_l: Vec::new(),
            buffer_r: Vec::new(),
            write_head: 0,
        }
    }

    fn effective_delay_secs(&self) -> f32 {
        if self.bpm_sync {
            let beat_secs = 60.0 / self.bpm;
            (beat_secs * DelaySubdivision::from_index(self.subdivision).multiplier())
                .min(MAX_DELAY_SECS)
        } else {
            self.delay_time.clamp(0.001, MAX_DELAY_SECS)
        }
    }

    fn delay_samples(&self) -> usize {
        (self.effective_delay_secs() * self.sample_rate) as usize
    }
}

impl PedalNode for DelayNode {
    fn prepare(&mut self, sample_rate: f32, _max_buffer_size: usize) {
        self.sample_rate = sample_rate;
        let buf_size = (sample_rate * MAX_DELAY_SECS) as usize + 1;
        self.buffer_l = vec![0.0; buf_size];
        self.buffer_r = vec![0.0; buf_size];
        self.write_head = 0;
        self.ping = false;
    }

    fn process(&mut self, data: &mut [f32]) {
        if self.bypass { return; }

        let buf_len = self.buffer_l.len();
        if buf_len == 0 { return; }

        let is_stereo = data.len() % 2 == 0;
        let frame_count = if is_stereo { data.len() / 2 } else { data.len() };
        let delay_samples = self.delay_samples().min(buf_len - 1);

        for i in 0..frame_count {
            let (dry_l, dry_r) = if is_stereo {
                (data[i * 2], data[i * 2 + 1])
            } else {
                (data[i], data[i])
            };

            let read = (self.write_head + buf_len - delay_samples) % buf_len;

            let (wet_l, wet_r) = if self.ping_pong {
                if self.ping { (self.buffer_l[read], 0.0) }
                else         { (0.0, self.buffer_r[read]) }
            } else {
                (self.buffer_l[read], self.buffer_r[read])
            };

            if self.ping_pong {
                if self.ping {
                    self.buffer_l[self.write_head] = dry_l + wet_r * self.feedback;
                    self.buffer_r[self.write_head] = 0.0;
                } else {
                    self.buffer_l[self.write_head] = 0.0;
                    self.buffer_r[self.write_head] = dry_r + wet_l * self.feedback;
                }
                self.ping = !self.ping;
            } else {
                self.buffer_l[self.write_head] = dry_l + wet_l * self.feedback;
                self.buffer_r[self.write_head] = dry_r + wet_r * self.feedback;
            }

            self.write_head = (self.write_head + 1) % buf_len;

            let out_l = dry_l * (1.0 - self.mix) + wet_l * self.mix;
            let out_r = dry_r * (1.0 - self.mix) + wet_r * self.mix;

            if is_stereo {
                data[i * 2]     = out_l;
                data[i * 2 + 1] = out_r;
            } else {
                data[i] = (out_l + out_r) * 0.5;
            }
        }
    }

    fn name(&self) -> &str { "Delay" }
    fn bypass(&self) -> bool { self.bypass }
    fn set_bypass(&mut self, bypass: bool) { self.bypass = bypass; }

    fn set_param(&mut self, param: usize, value: f32) {
        match param {
            0 => self.delay_time = value.clamp(0.001, MAX_DELAY_SECS),
            1 => self.feedback = value.clamp(0.0, 0.99),
            2 => self.mix = value.clamp(0.0, 1.0),
            3 => self.bpm_sync = value > 0.5,
            4 => self.bpm = value.clamp(20.0, 300.0),
            5 => self.subdivision = value,
            6 => self.ping_pong = value > 0.5,
            _ => {}
        }
    }
}
