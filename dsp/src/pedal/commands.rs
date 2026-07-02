use crate::pedal::PedalNode;

pub enum PedalChainCommand {
    // pedal management
    AddPedal { pedal: Box<dyn PedalNode> },
    InsertPedal { index: usize, pedal: Box<dyn PedalNode> },
    RemovePedal { index: usize },
    SwapPedals { a: usize, b: usize },

    // bypass
    SetBypass { index: usize, bypass: bool },
    ToggleBypass { index: usize },
    BypassAll,
    UnbypassAll,

    // parameters
    GetParamRef { index: usize, param: usize, value: f32 },

    // presets
    LoadPreset { pedals: Vec<Box<dyn PedalNode>> },
    ClearChain,
}

