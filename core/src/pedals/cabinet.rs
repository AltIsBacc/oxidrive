use oxidrive_dsp::pedal::PedalNode;
use rustfft::{FftPlanner, num_complex::Complex32};

pub struct CabinetNode {
    bypass: bool,
    ir: Vec<f32>,

    fft_size: usize,
    ir_spectrum: Vec<Complex32>,
    overlap: Vec<f32>,
    scratch: Vec<Complex32>,
    planner: FftPlanner<f32>,
}

impl CabinetNode {
    pub fn new(ir: Vec<f32>) -> Self {
        Self {
            bypass: false,
            ir,
            fft_size: 0,
            ir_spectrum: Vec::new(),
            overlap: Vec::new(),
            scratch: Vec::new(),
            planner: FftPlanner::new(),
        }
    }

    pub fn set_ir(&mut self, ir: Vec<f32>) {
        self.ir = ir;
        if self.fft_size > 0 {
            self.compute_ir_spectrum();
        }
    }

    fn compute_ir_spectrum(&mut self) {
        if self.ir.is_empty() || self.fft_size == 0 { return; }

        let fft = self.planner.plan_fft_forward(self.fft_size);

        let mut ir_padded: Vec<Complex32> = self.ir.iter()
            .take(self.fft_size)
            .map(|&s| Complex32::new(s, 0.0))
            .collect();
        ir_padded.resize(self.fft_size, Complex32::new(0.0, 0.0));

        fft.process(&mut ir_padded);
        self.ir_spectrum = ir_padded;
    }
}

impl PedalNode for CabinetNode {
    fn prepare(&mut self, _sample_rate: u32, max_buffer_size: usize) {
        let min_size = self.ir.len() + max_buffer_size;
        self.fft_size = min_size.next_power_of_two();
        self.overlap = vec![0.0; self.fft_size];
        self.scratch = vec![Complex32::new(0.0, 0.0); self.fft_size];
        // cache plans now so process() never allocates
        self.planner.plan_fft_forward(self.fft_size);
        self.planner.plan_fft_inverse(self.fft_size);
        self.compute_ir_spectrum();
    }

    fn process(&mut self, data: &mut [f32]) {
        if self.bypass || self.ir.is_empty() || self.ir_spectrum.is_empty() { return; }

        let block_size = data.len();
        let fft = self.planner.plan_fft_forward(self.fft_size);
        let ifft = self.planner.plan_fft_inverse(self.fft_size);

        // fill scratch in-place
        for (i, s) in self.scratch.iter_mut().enumerate() {
            s.re = if i < block_size { data[i] } else { 0.0 };
            s.im = 0.0;
        }

        fft.process(&mut self.scratch);

        for (x, h) in self.scratch.iter_mut().zip(self.ir_spectrum.iter()) {
            *x *= h;
        }

        ifft.process(&mut self.scratch);

        let norm = 1.0 / self.fft_size as f32;
        for i in 0..block_size {
            data[i] = (self.scratch[i].re * norm) + self.overlap[i];
        }

        self.overlap.fill(0.0);
        for i in block_size..self.fft_size {
            self.overlap[i - block_size] += self.scratch[i].re * norm;
        }
    }

    fn name(&self) -> &str { "Cabinet" }
    fn bypass(&self) -> bool { self.bypass }
    fn set_bypass(&mut self, bypass: bool) { self.bypass = bypass; }

    fn set_param(&mut self, param: usize, _value: f32) {
        match param {
            _ => {}
        }
    }
}

