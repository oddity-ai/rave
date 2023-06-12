pub trait Format: Copy + Clone + PartialEq + Eq {
    const ID: &'static str;
    const NUM_CHANNELS: usize;

    type T;
    type Data;
}

pub struct Plane<T> {
    pub data: Vec<T>,
    pub stride: usize,
}

pub struct Planar<T, const NUM_CHANNELS: usize> {
    pub planes: [Plane<T>; NUM_CHANNELS],
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Rgb24;

impl Format for Rgb24 {
    const ID: &'static str = "rgb24";
    const NUM_CHANNELS: usize = 3;

    type T = u8;
    type Data = Plane<Self::T>;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Yuv420p;

impl Format for Yuv420p {
    const ID: &'static str = "yuv420";
    const NUM_CHANNELS: usize = 3;

    type T = u8;
    type Data = Planar<Self::T, { Self::NUM_CHANNELS }>;
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
