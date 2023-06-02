use crate::device::Device;
use crate::format::Format;

pub struct Frame<D: Device, F: Format> {
    // TODO
    _phantom: std::marker::PhantomData<(D, F)>,
}

pub trait Dimensions {
    fn dims() -> (usize, usize);
}

impl<D: Device, F: Format> Dimensions for Frame<D, F> {
    fn dims() -> (usize, usize) {
        todo!()
    }
}

pub trait Channels {
    fn num_channels() -> usize;
}

impl<D: Device, F: Format> Channels for Frame<D, F> {
    fn num_channels() -> usize {
        F::NUM_CHANNELS
    }
}
