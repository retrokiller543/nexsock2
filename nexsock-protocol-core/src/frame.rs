use crate::traits::MessageBody;
use bincode::{Decode, Encode};

#[derive(Debug, /*Default, Clone, */ PartialEq, Eq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct Frame<const N: usize, T: MessageBody> {
    header: [u8; N],
    body: T,
}

impl<const N: usize, T: MessageBody> Frame<N, T> {
    pub fn new(header: [u8; N], body: T) -> Self {
        Self { header, body }
    }

    pub fn header(&self) -> [u8; N] {
        self.header
    }

    pub fn body(&self) -> &T {
        &self.body
    }
}
