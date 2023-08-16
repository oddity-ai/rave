use bytes::Bytes;

pub trait Codec {
    const ID: &'static str;

    type Data;
}

pub struct H264;

impl Codec for H264 {
    const ID: &'static str = "h264";

    type Data = Bytes;
}
