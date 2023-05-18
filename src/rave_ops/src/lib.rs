use rave_types::{Frame, Format};

// TODO: autoformat
pub trait FrameOp<FrameFormat1, FrameFormat2>: Fn(&Frame<FrameFormat1>) -> Frame<FrameFormat2> where FrameFormat1: Format, FrameFormat2: Format {}