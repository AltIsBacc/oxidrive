
pub trait Normalized {
    fn to_bool(self) -> bool;
}

impl Normalized for f32 {
    fn to_bool(self) -> bool {
        self > 0.5
    }
}

