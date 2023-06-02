use crate::device::Device;
use crate::format::Format;
use crate::frame::Frame;

pub type DecodeResult<Device, Format, Error> =
    std::result::Result<Option<Frame<Device, Format>>, Error>;

pub trait Decode {
    type Device: Device;
    type Format: Format;
    type Error;

    fn decode(&mut self, packet: &[u8]) -> DecodeResult<Self::Device, Self::Format, Self::Error>;
}
