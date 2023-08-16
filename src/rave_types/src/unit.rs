// TODO: generalize Unit type over codec
use crate::codec::Codec;

pub struct Unit<C: Codec> {
    pub data: C::Data,
}

impl<C: Codec> Unit<C> {
    pub fn new(data: C::Data) -> Self {
        Self { data }
    }

    pub fn into_data(self) -> C::Data {
        self.data
    }
}
