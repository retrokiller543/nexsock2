#![cfg(feature = "simd")]
#![allow(unsafe_code)]

use crate::constants::HEADER_SIZE;
use crate::header::Header;
use crate::message_flags::MessageFlags;
use crate::traits::header::{HeaderDeserializer, HeaderSerializer};
use bytes::{Buf, Bytes};
use std::simd::SupportedLaneCount;

pub struct Aarch64NeonHeaderParser;

#[cfg(target_arch = "aarch64")]
impl HeaderSerializer for Aarch64NeonHeaderParser {
    #[inline]
    fn serialize(header: &Header) -> [u8; HEADER_SIZE] {
        use std::arch::aarch64::*;

        unsafe {
            let mut buffer = std::mem::MaybeUninit::<[u8; HEADER_SIZE]>::uninit();
            let buf_ptr = buffer.as_mut_ptr() as *mut u8;

            *buf_ptr = ((header.id & Header::LAST_SIX_BITS) << 2) | (header.version & Header::LAST_TWO_BITS);

            let flags_be = (*header.flags).to_be();
            std::ptr::write_unaligned(buf_ptr.add(1) as *mut u16, flags_be);

            let payload_be = header.payload_len.to_be();
            std::ptr::write_unaligned(buf_ptr.add(3) as *mut u32, payload_be);

            let seq_be = header.sequence_number.to_be();

            let seq_neon = vld1_u8((&seq_be as *const u64) as *const u8);
            vst1_u8(buf_ptr.add(7), seq_neon);

            buffer.assume_init()
        }
    }
}

#[cfg(target_arch = "aarch64")]
impl HeaderDeserializer for Aarch64NeonHeaderParser {
    #[inline]
    fn parse(buf: &[u8]) -> Option<Header> {
        use std::arch::aarch64::*;

        if buf.len() < HEADER_SIZE {
            return None;
        }

        unsafe {
            // Extract first byte for id and version
            let first_byte = *buf.as_ptr();
            let id = (first_byte & Header::LAST_SIX_BITS) >> 2;
            let version = first_byte & Header::LAST_TWO_BITS;

            // Read flags (2 bytes) - avoid misalignment by reading bytes directly
            let flags = u16::from_be_bytes([*buf.as_ptr().add(1), *buf.as_ptr().add(2)]);

            // Read payload length (4 bytes) - avoid misalignment by reading bytes directly
            let payload_len = u32::from_be_bytes([
                *buf.as_ptr().add(3),
                *buf.as_ptr().add(4),
                *buf.as_ptr().add(5),
                *buf.as_ptr().add(6)
            ]);

            // Use NEON for sequence number (8 bytes)
            let seq_ptr = buf.as_ptr().add(7);
            let seq_neon = vld1_u8(seq_ptr);

            // Convert sequence number back to native endianness
            let mut seq_bytes = [0u8; 8];
            vst1_u8(seq_bytes.as_mut_ptr(), seq_neon);
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
    use crate::header::optimized::OptimizedHeaderParser;
    use crate::header::standard::StandardHeaderParser;
    use crate::header::tests::{test_deserializer, test_serializer};

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_aarch64_neon_serialize() {
        test_serializer::<Aarch64NeonHeaderParser>()
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_aarch64_neon_deserialize() {
        test_deserializer::<Aarch64NeonHeaderParser>()
    }
}