pub mod header;

use bincode::{Decode, Encode};
use crate::frame::Frame;

pub trait MessageBody: Encode + Decode<()> {
    fn to_frame<const N: usize>(self, header: [u8; N]) -> Frame<N, Self> {
        Frame::new(header, self)
    }
}

impl MessageBody for () {}