use rave_types::device::Device;
use rave_types::format::Format;
use rave_types::frame::Frame;

pub trait FrameOp<Device1, Format1, Device2, Format2>:
    Fn(&Frame<Device1, Format1>) -> Frame<Device2, Format2>
where
    Format1: Format,
    Device1: Device,
    Format2: Format,
    Device2: Device,
{
}
