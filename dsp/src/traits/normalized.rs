
pub trait Normalized {
    fn to_bool(self) -> bool;
}

impl Normalized for f32 {
    fn to_bool(self) -> bool {
        self.clamp(0.0, 1.0) > 0.5
    }
}

