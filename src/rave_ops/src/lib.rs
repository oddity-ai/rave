use rave_types::{Format, Frame};

pub trait FrameOp<FrameFormat1, FrameFormat2>:
    Fn(&Frame<FrameFormat1>) -> Frame<FrameFormat2>
where
    FrameFormat1: Format,
    FrameFormat2: Format,
{
}
