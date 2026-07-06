use std::any::Any;

use crate::pedal::BoxedPedal;

pub enum ChainCommand {
    AddPedal(BoxedPedal),
}

pub enum ChainUpdate {
    CommandAcknowledged,
    PedalReady,
    FreeObject(Box<dyn Any + Send>),
}

