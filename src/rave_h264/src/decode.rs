use rave_types::decode::Decode;
use rave_types::device::Local;
use rave_types::format::Yuv420p;
use rave_types::frame::Frame;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Decoder {
    inner: openh264::decoder::Decoder,
}

impl Decoder {
    pub fn new() -> Result<Self> {
        Ok(Decoder {
            inner: openh264::decoder::Decoder::new()?,
        })
    }
}

impl Decode for Decoder {
    type Device = Local;
    type Format = Yuv420p;
    type Error = Error;

    fn decode(&mut self, packet: &[u8]) -> Result<Option<Frame<Self::Device, Self::Format>>> {
        self.inner
            .decode(packet)
            .map(|out| out.map(convert_frame))
            .map_err(|err| err.into())
    }
}

fn convert_frame(frame: openh264::decoder::DecodedYUV) -> Frame<Local, Yuv420p> {
    todo!()
}
