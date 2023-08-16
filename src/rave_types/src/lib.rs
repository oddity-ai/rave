// TODO: extract frame stuff
// TODO: extract cuda stuff (?)
// TODO: use crate::error::Error;
// TODO: feature flag for cuda stuff

pub mod codec;
pub mod decode;
pub mod device;
pub mod encode;
pub mod format;
pub mod frame;
pub mod unit;

pub use codec::{Codec, H264};
pub use decode::Decode;
pub use device::{Cuda, Device, Local};
pub use encode::Encode;
pub use format::{Format, Planar, Plane, Rgb24, Yuv420p};
pub use frame::{Frame, RgbFrame, Yuv420pFrame};
pub use unit::Unit;
