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
        todo!()
        // match self.inner.encode(todo!()) {
        //     Ok(output) => Ok((0..output.num_layers())
        //         .map(|layer_index| output.layer(layer_index).unwrap())
        //         .map(|layer| {
        //             (0..layer.nal_count())
        //                 .map(|nal_unit_index| layer.nal_unit(nal_unit_index).unwrap())
        //                 .map(|nal_unit| nal_unit.to_vec())
        //         })
        //         .flatten()
        //         .map(|nal_unit| Unit::<H264>::new(nal_unit))
        //         .collect()),
        //     Err(err) => Err(err.into()),
        // }
    }
}
