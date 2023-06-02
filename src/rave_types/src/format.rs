pub trait Format: Copy + Clone + PartialEq + Eq {
    const ID: &'static str;
    const NUM_CHANNELS: usize;
    const PLANAR: bool;
    type T;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Yuv420p;

impl Format for Yuv420p {
    const ID: &'static str = "yuv420";
    const NUM_CHANNELS: usize = 3;
    const PLANAR: bool = true;
    type T = u8;
}

macro_rules! impl_display_for {
    ($f:ty) => {
        impl std::fmt::Display for $f {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", Self::ID)
            }
        }
    };
}

impl_display_for!(Yuv420p);
