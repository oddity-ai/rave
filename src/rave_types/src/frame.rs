use crate::device::{Device, Local};
use crate::format::{Format, Rgb24, Yuv420p};

// convenience aliases
pub type Yuv420pFrame = Frame<Local, Yuv420p>;
pub type RgbFrame = Frame<Local, Rgb24>;

pub struct Frame<D: Device, F: Format> {
    pub data: D::Container<F::Data>,
    pub dims: (usize, usize),
}

impl<D: Device, F: Format> Frame<D, F> {
    pub fn new(data: D::Container<F::Data>, dims: (usize, usize)) -> Self {
        Self { data, dims }
    }

    #[inline(always)]
    pub const fn num_channels() -> usize {
        F::NUM_CHANNELS
    }
}

pub trait Dimensions {
    fn dims(&self) -> (usize, usize);
}

impl<D: Device, F: Format> Dimensions for Frame<D, F> {
    fn dims(&self) -> (usize, usize) {
        self.dims
    }
}
