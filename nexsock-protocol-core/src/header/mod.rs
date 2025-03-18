use crate::constants::HEADER_SIZE;
use crate::error::ProtocolResult;
#[cfg(target_arch = "aarch64")]
use crate::header::simd::Aarch64NeonHeaderParser;
#[cfg(not(feature = "simd"))]
use crate::header::standard::StandardHeaderParser;
#[cfg(not(target_arch = "aarch64"))]
use crate::header::standard::StandardHeaderParser;
use crate::message_flags::MessageFlags;
use crate::traits::header::{HeaderDeserializer, HeaderParser, HeaderSerializer};
use bytes::Bytes;
use futures::AsyncRead;
#[cfg(feature = "simd")]
use optimized::OptimizedHeaderParser;

pub mod optimized;
pub mod simd;
pub mod standard;

/// Default parser combination based on configuration
pub struct DefaultHeaderParser;

impl HeaderParser for DefaultHeaderParser {
    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
            type Serializer = crate::header::simd::Aarch64NeonHeaderParser;
        } else {
            type Serializer = crate::header::standard::StandardHeaderParser;
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
            type Deserializer = crate::header::simd::Aarch64NeonHeaderParser;
        } else {
            type Deserializer = crate::header::optimized::OptimizedHeaderParser;
        }
    }
}

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

    #[inline(always)]
    pub fn new(
        id: u8,
        version: u8,
        flags: MessageFlags,
        payload_len: u32,
        sequence_number: u64,
    ) -> Self {
        Self {
            id,
            version,
            flags,
            payload_len,
            sequence_number,
        }
    }

    #[inline(always)]
    pub fn to_bytes<S: HeaderSerializer>(&self) -> [u8; HEADER_SIZE] {
        S::serialize(self)
    }

    #[inline(always)]
    pub fn parse<P: HeaderDeserializer>(bytes: &[u8]) -> Option<Self> {
        P::parse(bytes)
    }

    #[inline(always)]
    pub fn parse_bytes<P: HeaderDeserializer>(bytes: &mut Bytes) -> Option<Self> {
        P::parse_bytes(bytes)
    }

    #[inline(always)]
    pub async fn read_header<P: HeaderDeserializer, R: AsyncRead + Unpin>(
        reader: &mut R,
    ) -> ProtocolResult<Self> {
        P::read_header(reader).await
    }

    #[inline(always)]
    pub fn id(&self) -> u8 {
        self.id
    }

    #[inline(always)]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[inline(always)]
    pub fn flags(&self) -> MessageFlags {
        self.flags
    }

    #[inline(always)]
    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    #[inline(always)]
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::header::standard::StandardHeaderParser;

    const HEADER_BYTES: [u8; HEADER_SIZE] = [6, 0, 9, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1];

    pub(crate) fn test_serializer<S: HeaderSerializer>() {
        let version = 2;
        let id = 1;
        let flags = MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD;
        let payload_len = 0x200;
        let sequence_number = 1;

        let original_header = Header::new(id, version, flags, payload_len, sequence_number);
        let header_bytes = S::serialize(&original_header);

        assert_eq!(header_bytes.len(), HEADER_SIZE);
        assert_eq!(header_bytes[..], HEADER_BYTES[..]);
    }

    pub(crate) fn test_deserializer<D: HeaderDeserializer>() {
        let header = D::parse(&HEADER_BYTES);

        assert!(header.is_some());
        let header = header.unwrap();

        assert_eq!(header.id(), 1);
        assert_eq!(header.version(), 2);
        assert_eq!(
            header.flags(),
            MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD
        );
        assert_eq!(header.payload_len(), 0x200);
        assert_eq!(header.sequence_number(), 1);
    }

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
            0x06, // id and version combined
            0x00, 0x00, // flags (assumed value)
            0x00, 0x00, 0x02, 0x00, // payload_len (0x200) - in big endian
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, // sequence_number (1) - in big endian
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
        let recovered_header =
            Header::parse::<StandardHeaderParser>(&mut Bytes::from_owner(bytes)).unwrap();

        assert_eq!(recovered_header.id, id);
        assert_eq!(recovered_header.version, version);
        assert_eq!(recovered_header.flags, flags);
        assert_eq!(recovered_header.payload_len, payload_len);
        assert_eq!(recovered_header.sequence_number, sequence_number);
    }
}
