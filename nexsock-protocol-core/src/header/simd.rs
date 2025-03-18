#![cfg(feature = "simd")]
#![allow(unsafe_code)]

use crate::constants::HEADER_SIZE;
use crate::header::Header;
use crate::message_flags::MessageFlags;
use crate::traits::header::{HeaderDeserializer, HeaderSerializer};
use bytes::{Buf, Bytes};
use std::simd::SupportedLaneCount;

/// A heavily optimized parser for `aarch64` targets, it's not recommended you need to get every last bit of performance
/// by leveraging aarch64 neon.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub struct Aarch64NeonHeaderParser;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
impl HeaderSerializer for Aarch64NeonHeaderParser {
    #[inline]
    fn serialize(header: &Header) -> [u8; HEADER_SIZE] {
        use std::arch::aarch64::*;

        unsafe {
            // First create a 128-bit NEON vector and fill it with our data
            // This allows us to do a single write at the end
            let mut tmp_buf = [0u8; 16]; // Use 16 bytes for alignment

            // Pack fields with proper endianness
            tmp_buf[0] = ((header.id & Header::LAST_SIX_BITS) << 2)
                | (header.version & Header::LAST_TWO_BITS);

            let flags_be = (*header.flags).to_be();
            std::ptr::write_unaligned(tmp_buf.as_mut_ptr().add(1) as *mut u16, flags_be);

            let payload_be = header.payload_len.to_be();
            std::ptr::write_unaligned(tmp_buf.as_mut_ptr().add(3) as *mut u32, payload_be);

            let seq_be = header.sequence_number.to_be();
            std::ptr::write_unaligned(tmp_buf.as_mut_ptr().add(7) as *mut u64, seq_be);

            // Load the prepared data into a NEON register
            let neon_data = vld1q_u8(tmp_buf.as_ptr());

            // Write to final buffer in one NEON operation
            let mut buffer = std::mem::MaybeUninit::<[u8; HEADER_SIZE]>::uninit();
            vst1q_u8(buffer.as_mut_ptr() as *mut u8, neon_data);

            buffer.assume_init()
        }
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
impl HeaderDeserializer for Aarch64NeonHeaderParser {
    #[inline]
    fn parse(buf: &[u8]) -> Option<Header> {
        use std::arch::aarch64::*;

        if buf.len() < HEADER_SIZE {
            return None;
        }

        unsafe {
            let neon_data = vld1q_u8(buf.as_ptr());

            let mut tmp_buf = [0u8; HEADER_SIZE];
            vst1q_u8(tmp_buf.as_mut_ptr(), neon_data);

            let first_byte = tmp_buf[0];
            let id = (first_byte >> 2) & Header::LAST_SIX_BITS;
            let version = first_byte & Header::LAST_TWO_BITS;

            let flags = u16::from_be_bytes([tmp_buf[1], tmp_buf[2]]);

            let payload_len = u32::from_be_bytes([tmp_buf[3], tmp_buf[4], tmp_buf[5], tmp_buf[6]]);

            let sequence_number = u64::from_be_bytes([
                tmp_buf[7],
                tmp_buf[8],
                tmp_buf[9],
                tmp_buf[10],
                tmp_buf[11],
                tmp_buf[12],
                tmp_buf[13],
                tmp_buf[14],
            ]);

            Some(Header::new(
                id,
                version,
                MessageFlags::from(flags),
                payload_len,
                sequence_number,
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
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    fn test_aarch64_neon_serialize() {
        test_serializer::<Aarch64NeonHeaderParser>()
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    fn test_aarch64_neon_deserialize() {
        test_deserializer::<Aarch64NeonHeaderParser>()
    }
}
