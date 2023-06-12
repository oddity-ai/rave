use crate::codec::Codec;
use crate::device::Device;
use crate::format::Format;
use crate::frame::Frame;
use crate::unit::Unit;

pub type DecodeResult<Device, Format, Error> =
    std::result::Result<Option<Frame<Device, Format>>, Error>;

pub trait Decode {
    type Device: Device;
    type Codec: Codec;
    type Format: Format;
    type Error;

    fn decode(
        &mut self,
        unit: Unit<Self::Codec>,
    ) -> DecodeResult<Self::Device, Self::Format, Self::Error>;
}
