#![cfg(feature = "simd")]
#![allow(unsafe_code)]

use std::simd::{Simd, LaneCount, SupportedLaneCount};
use bytes::{Buf, Bytes, BytesMut};
use crate::constants::HEADER_SIZE;
use crate::message_flags::MessageFlags;
use crate::header::Header;
use crate::error::ProtocolResult;
use crate::traits::header::HeaderParser;

/// SIMD-accelerated header parser
pub struct SimdHeaderParser;

impl HeaderParser for SimdHeaderParser {
    fn parse(buf: &[u8]) -> Option<Header> {
        if buf.len() < HEADER_SIZE {
            return None;
        }

        unsafe {
            // Read the first byte for id and version
            let first_byte = buf[0];
            let id = (first_byte & Header::LAST_SIX_BITS) >> 2;
            let version = first_byte & Header::LAST_TWO_BITS;

            // Use direct memory access for big-endian multi-byte values
            // Flags (u16 - 2 bytes)
            let flags_bytes = std::ptr::read_unaligned(buf.as_ptr().add(1) as *const [u8; 2]);
            let flags = u16::from_be_bytes(flags_bytes);

            // Payload length (u32 - 4 bytes)
            let payload_bytes = std::ptr::read_unaligned(buf.as_ptr().add(3) as *const [u8; 4]);
            let payload_len = u32::from_be_bytes(payload_bytes);

            // Sequence number (u64 - 8 bytes)
            let seq_bytes = std::ptr::read_unaligned(buf.as_ptr().add(7) as *const [u8; 8]);
            let sequence_number = u64::from_be_bytes(seq_bytes);

            Some(Header::new(
                id,
                version,
                MessageFlags::from(flags),
                payload_len,
                sequence_number
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_header_parser_optimized() {
        let version = 2;
        let id = 1;
        let flags = MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD;
        let payload_len = 0x200;
        let sequence_number = 1;

        let original_header = Header::new(id, version, flags, payload_len, sequence_number);
        let header_bytes = original_header.to_bytes();
        
        let parsed_header = SimdHeaderParser::parse(&header_bytes).unwrap();
        
        assert_eq!(parsed_header.id(), id);
        assert_eq!(parsed_header.version(), version);
        assert_eq!(parsed_header.flags(), flags);
        assert_eq!(parsed_header.payload_len(), payload_len);
        assert_eq!(parsed_header.sequence_number(), sequence_number);
    }

    #[test]
    fn test_simd_header_parser() {
        let version = 2;
        let id = 1;
        let flags = MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD;
        let payload_len = 0x200;
        let sequence_number = 1;

        let original_header = Header::new(id, version, flags, payload_len, sequence_number);
        let header_bytes = original_header.to_bytes();
        
        let parsed_header = SimdHeaderParser::parse(&header_bytes).unwrap();

        assert_eq!(parsed_header.id(), id);
        assert_eq!(parsed_header.version(), version);
        assert_eq!(parsed_header.flags(), flags);
        assert_eq!(parsed_header.payload_len(), payload_len);
        assert_eq!(parsed_header.sequence_number(), sequence_number);
    }

    #[test]
    fn test_simd_with_bytes() {
        let header = Header::new(5, 1, MessageFlags::HAS_PAYLOAD, 100, 123456);
        let bytes = header.to_bytes();

        let parsed = SimdHeaderParser::parse_bytes(&mut Bytes::from_owner(bytes)).unwrap();
        
        assert_eq!(parsed.id(), 5);
        assert_eq!(parsed.version(), 1);
        assert_eq!(parsed.flags(), MessageFlags::HAS_PAYLOAD);
        assert_eq!(parsed.payload_len(), 100);
        assert_eq!(parsed.sequence_number(), 123456);
        
        assert_eq!(bytes.len(), 0);
    }

    #[tokio::test]
    async fn test_async_read_header() {
        let header = Header::new(3, 2, MessageFlags::ENCRYPTED, 250, 9999);
        let header_bytes = header.to_bytes();
        
        use crate::transport::tests::MockReader;
        let reader = MockReader::new(header_bytes.to_vec());
        
        let mut reader = reader;
        let parsed = SimdHeaderParser::read_header(&mut reader).await.unwrap();
        
        assert_eq!(parsed.id(), 3);
        assert_eq!(parsed.version(), 2);
        assert_eq!(parsed.flags(), MessageFlags::ENCRYPTED);
        assert_eq!(parsed.payload_len(), 250);
        assert_eq!(parsed.sequence_number(), 9999);
    }
}