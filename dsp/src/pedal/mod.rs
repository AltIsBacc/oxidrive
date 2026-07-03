use crate::engine::buffer::AudioBuffer;

pub mod chain;

pub trait PedalNode: Send {
    fn prepare(&mut self, sample_rate: u32, buffer_size: usize);
    fn process(&mut self, data: &AudioBuffer<'_, f32>);

    fn name(&self) -> &str;
    fn bypass(&self) -> bool;

    fn set_bypass(&mut self, bypass: bool);
    fn set_param_raw(&mut self, param: u32, value: f32);
}

pub trait TypedPedalNode: PedalNode {
    fn set_param<P: Into<u32>>(&mut self, param: P, value: f32);
}

impl<T: PedalNode + ?Sized> TypedPedalNode for T {
    fn set_param<P: Into<u32>>(&mut self, param: P, value: f32) {
        self.set_param_raw(param.into(), value);
    }
}

type BoxedPedal = Box<dyn PedalNode>;

