use crate::codec::Codec;
use crate::device::Device;
use crate::format::Format;
use crate::frame::Frame;
use crate::unit::Unit;

pub trait Encode {
    type Device: Device;
    type Codec: Codec;
    type Format: Format;
    type Error;

    fn encode(
        &mut self,
        frame: Frame<Self::Device, Self::Format>,
    ) -> Result<Vec<Unit<Self::Codec>>, Self::Error>;
}
