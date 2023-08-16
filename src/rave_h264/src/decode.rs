use rave_types::codec::H264;
use rave_types::decode::Decode;
use rave_types::device::Local;
use rave_types::format::{Planar, Plane, Yuv420p};
use rave_types::frame::Yuv420pFrame;
use rave_types::unit::Unit;

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
    type Codec = H264;
    type Format = Yuv420p;
    type Error = Error;

    fn decode(&mut self, unit: Unit<H264>) -> Result<Option<Yuv420pFrame>> {
        match self.inner.decode(unit.data.as_ref()) {
            Ok(frame) => Ok(frame.map(convert_frame)),
            Err(err) => Err(err.into()),
        }
    }
}

fn convert_frame(frame: openh264::decoder::DecodedYUV) -> Yuv420pFrame {
    let (stride_y, stride_u, stride_v) = frame.strides_yuv();
    Yuv420pFrame::new(
        Planar {
            planes: [
                Plane {
                    data: frame.y_with_stride().to_vec(),
                    stride: stride_y,
                },
                Plane {
                    data: frame.u_with_stride().to_vec(),
                    stride: stride_u,
                },
                Plane {
                    data: frame.v_with_stride().to_vec(),
                    stride: stride_v,
                },
            ],
        },
        frame.dimension_rgb(),
    )
}
