pub trait Codec {
    const ID: &'static str;
}

pub struct H264;

impl Codec for H264 {
    const ID: &'static str = "h264";
}
