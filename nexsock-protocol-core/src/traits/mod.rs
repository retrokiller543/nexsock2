pub mod header;

use crate::frame::Frame;
use bincode::{Decode, Encode};

pub trait MessageBody: Encode + Decode<()> {
    fn to_frame<const N: usize>(self, header: [u8; N]) -> Frame<N, Self> {
        Frame::new(header, self)
    }
}

impl MessageBody for () {}