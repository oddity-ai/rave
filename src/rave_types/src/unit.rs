// TODO: generalize Unit type over codec
use crate::codec::Codec;

pub struct Unit<C: Codec> {
    pub data: Vec<u8>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Codec> Unit<C> {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            _phantom: Default::default(),
        }
    }
}
