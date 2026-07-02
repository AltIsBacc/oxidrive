
pub mod chain;
pub mod commands;

pub trait PedalNode: Send {
    fn prepare(&mut self, sample_rate: u32, buffer_size: usize);
    fn process(&mut self, data: &mut [f32]);

    fn name(&self) -> &str;
    fn bypass(&self) -> bool;

    fn set_bypass(&mut self, bypass: bool);
    fn set_param(&mut self, param: usize, value: f32);
}

