use crate::node::AudioNode;

pub enum ChainCommand {
    // pedal management
    AddPedal { index: usize, pedal: Box<dyn AudioNode> },
    RemovePedal { index: usize },
    SwapPedals { a: usize, b: usize },
    MovePedal { from: usize, to: usize },

    // bypass
    SetBypass { index: usize, bypass: bool },
    ToggleBypass { index: usize },
    BypassAll,
    UnbypassAll,

    // parameters
    SetParam { index: usize, param: usize, value: f32 },

    // presets
    LoadPreset { pedals: Vec<Box<dyn AudioNode>> },
    ClearChain,
}

