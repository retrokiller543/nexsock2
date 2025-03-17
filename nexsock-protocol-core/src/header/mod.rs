use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::AsyncRead;
use crate::constants::HEADER_SIZE;
use crate::error::ProtocolResult;
use crate::message_flags::MessageFlags;
use crate::traits::header::{HeaderDeserializer, HeaderSerializer};

pub mod simd;
pub mod standard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Header {
    id: u8,
    version: u8,
    flags: MessageFlags,
    payload_len: u32,
    sequence_number: u64,
}

impl Header {
    pub(crate) const LAST_SIX_BITS: u8 = 0x3F;
    pub(crate) const LAST_TWO_BITS: u8 = 0x03;

    pub fn new(id: u8, version: u8, flags: MessageFlags, payload_len: u32, sequence_number: u64) -> Self {
        Self { id, version, flags, payload_len, sequence_number }
    }

    /*pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buffer = [0u8; HEADER_SIZE];

        buffer[0] = ((self.id & Self::LAST_SIX_BITS) << 2) | (self.version & Self::LAST_TWO_BITS);

        buffer[1] = (*self.flags >> 8) as u8;
        buffer[2] = *self.flags as u8;

        buffer[3] = (self.payload_len >> 24) as u8;
        buffer[4] = (self.payload_len >> 16) as u8;
        buffer[5] = (self.payload_len >> 8) as u8;
        buffer[6] = self.payload_len as u8;

        buffer[7] = (self.sequence_number >> 56) as u8;
        buffer[8] = (self.sequence_number >> 48) as u8;
        buffer[9] = (self.sequence_number >> 40) as u8;
        buffer[10] = (self.sequence_number >> 32) as u8;
        buffer[11] = (self.sequence_number >> 24) as u8;
        buffer[12] = (self.sequence_number >> 16) as u8;
        buffer[13] = (self.sequence_number >> 8) as u8;
        buffer[14] = self.sequence_number as u8;
        
        buffer
    }*/

    pub fn to_bytes<S: HeaderSerializer>(&self) -> [u8; HEADER_SIZE] {
        S::serialize(self)
    }
    
    pub fn parse<P: HeaderDeserializer>(bytes: &[u8]) -> Option<Self> {
        P::parse(bytes)
    }

    pub fn parse_bytes<P: HeaderDeserializer>(bytes: &mut Bytes) -> Option<Self> {
        P::parse_bytes(bytes)
    }

    pub async fn read_header<P: HeaderDeserializer, R: AsyncRead + Unpin>(reader: &mut R) -> ProtocolResult<Self> {
        P::read_header(reader).await
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn flags(&self) -> MessageFlags {
        self.flags
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
}

#[cfg(test)]
mod tests {
    use crate::header::simd::SimdHeaderParser;
    use crate::header::standard::StandardHeaderParser;
    use super::*;

    #[test]
    fn test_to_bytes() {
        let version = 2;
        let id = 1;
        let flags = MessageFlags::NONE;
        let payload_len = 0x200;
        let sequence_number = 1;

        let header = Header::new(id, version, flags, payload_len, sequence_number);

        let header_bytes = header.to_bytes::<StandardHeaderParser>();
        
        let expected_bytes = [
            0x06,                         // id and version combined
            0x00, 0x00,                   // flags (assumed value)
            0x00, 0x00, 0x02, 0x00,       // payload_len (0x200) - in big endian
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01  // sequence_number (1) - in big endian
        ];

        assert_eq!(header_bytes.len(), HEADER_SIZE);
        assert_eq!(&header_bytes[..], &expected_bytes[..]);
    }

    #[test]
    fn test_roundtrip() {
        let version = 2;
        let id = 1;
        let flags = MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD;
        let payload_len = 0x200;
        let sequence_number = 1;

        let original_header = Header::new(id, version, flags, payload_len, sequence_number);
        let bytes = original_header.to_bytes::<StandardHeaderParser>();
        let recovered_header = Header::parse::<StandardHeaderParser>(&mut Bytes::from_owner(bytes)).unwrap();

        assert_eq!(recovered_header.id, id);
        assert_eq!(recovered_header.version, version);
        assert_eq!(recovered_header.flags, flags);
        assert_eq!(recovered_header.payload_len, payload_len);
        assert_eq!(recovered_header.sequence_number, sequence_number);
    }
}

