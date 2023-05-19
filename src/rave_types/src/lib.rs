// FIXME: these are just some ideas

// TODO: extract frame stuff
// TODO: extract cuda stuff (?)

// TODO: use crate::error::Error;

type Error = (); // TODO: dummy

type Result<T> = std::result::Result<T, Error>;

pub trait Format {
    type T;
    const NUM_CHANNELS: usize;
}

pub struct Frame<F: Format> {
    // TODO
    _phantom: std::marker::PhantomData<F>,
}

pub trait Dimensions {
    fn dims() -> (usize, usize);
}

impl<F: Format> Dimensions for Frame<F> {
    fn dims() -> (usize, usize) {
        todo!()
    }
}

pub trait Channels {
    fn num_channels() -> usize;
}

impl<F: Format> Channels for Frame<F> {
    fn num_channels() -> usize {
        F::NUM_CHANNELS
    }
}

// TODO: feature flag for cuda stuff

pub struct CudaFrame<F: Format> {
    // TODO
    _phantom: std::marker::PhantomData<F>,
}

impl<F: Format> Dimensions for CudaFrame<F> {
    fn dims() -> (usize, usize) {
        todo!()
    }
}

impl<F: Format> Channels for CudaFrame<F> {
    fn num_channels() -> usize {
        F::NUM_CHANNELS
    }
}
