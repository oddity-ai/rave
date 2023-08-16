use rave_types::codec::H264;
use rave_types::device::Local;
use rave_types::encode::Encode;
use rave_types::format::Yuv420p;
use rave_types::frame::Yuv420pFrame;
use rave_types::unit::Unit;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub type Config = openh264::encoder::EncoderConfig;

pub struct Encoder {
    inner: openh264::encoder::Encoder,
}

impl Encoder {
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Result<Self> {
        Ok(Self {
            inner: openh264::encoder::Encoder::with_config(config)?,
        })
    }
}

impl Encode for Encoder {
    type Device = Local;
    type Codec = H264;
    type Format = Yuv420p;
    type Error = Error;

    fn encode(&mut self, frame: Yuv420pFrame) -> Result<Vec<Unit<H264>>> {
        match self.inner.encode(&CompatibleYuv420pFrame::from(frame)) {
            Ok(output) => {
                let mut units = Vec::new();
                for layer_index in 0..output.num_layers() {
                    let layer = output.layer(layer_index).unwrap();
                    for nal_unit_index in 0..layer.nal_count() {
                        units.push(Unit::new(
                            layer.nal_unit(nal_unit_index).unwrap().to_vec().into(),
                        ));
                    }
                }
                Ok(units)
            }
            Err(err) => Err(err.into()),
        }
    }
}

pub struct CompatibleYuv420pFrame {
    inner: Yuv420pFrame,
}

impl From<Yuv420pFrame> for CompatibleYuv420pFrame {
    #[inline(always)]
    fn from(value: Yuv420pFrame) -> Self {
        CompatibleYuv420pFrame { inner: value }
    }
}

impl openh264::formats::YUVSource for CompatibleYuv420pFrame {
    #[inline]
    fn width(&self) -> i32 {
        self.inner.dims.0.try_into().unwrap()
    }

    #[inline]
    fn height(&self) -> i32 {
        self.inner.dims.1.try_into().unwrap()
    }

    #[inline]
    fn y(&self) -> &[u8] {
        &self.inner.data.planes[0].data
    }

    #[inline]
    fn u(&self) -> &[u8] {
        &self.inner.data.planes[1].data
    }

    #[inline]
    fn v(&self) -> &[u8] {
        &self.inner.data.planes[2].data
    }

    #[inline]
    fn y_stride(&self) -> i32 {
        self.inner.dims.0.try_into().unwrap()
    }

    #[inline]
    fn u_stride(&self) -> i32 {
        self.inner.dims.0.try_into().unwrap()
    }

    #[inline]
    fn v_stride(&self) -> i32 {
        self.inner.dims.0.try_into().unwrap()
    }
}
