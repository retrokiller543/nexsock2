use crate::traits::header::HeaderSerializer;
use crate::{
    constants::HEADER_SIZE, header::Header, message_flags::MessageFlags,
    traits::header::HeaderDeserializer,
};

pub struct StandardHeaderParser;

impl HeaderDeserializer for StandardHeaderParser {
    #[inline]
    fn parse(bytes: &[u8]) -> Option<Header> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }

        let id_version = bytes[0];
        let id = (id_version & Header::LAST_SIX_BITS) >> 2;
        let version = id_version & Header::LAST_TWO_BITS;

        let flags = ((bytes[1] as u16) << 8) | (bytes[2] as u16);

        let payload_len = ((bytes[3] as u32) << 24)
            | ((bytes[4] as u32) << 16)
            | ((bytes[5] as u32) << 8)
            | (bytes[6] as u32);

        let sequence_number = ((bytes[7] as u64) << 56)
            | ((bytes[8] as u64) << 48)
            | ((bytes[9] as u64) << 40)
            | ((bytes[10] as u64) << 32)
            | ((bytes[11] as u64) << 24)
            | ((bytes[12] as u64) << 16)
            | ((bytes[13] as u64) << 8)
            | (bytes[14] as u64);

        Some(Header::new(
            id,
            version,
            MessageFlags::from(flags),
            payload_len,
            sequence_number,
        ))
    }
}

impl HeaderSerializer for StandardHeaderParser {
    #[inline]
    fn serialize(header: &Header) -> [u8; HEADER_SIZE] {
        unsafe {
            // Initialize buffer without zero-initialization overhead
            let mut buffer = std::mem::MaybeUninit::<[u8; HEADER_SIZE]>::uninit();
            let buf_ptr = buffer.as_mut_ptr() as *mut u8;

            // First byte: id and version packed together
            *buf_ptr = ((header.id & Header::LAST_SIX_BITS) << 2)
                | (header.version & Header::LAST_TWO_BITS);

            // For maximum performance on modern CPUs, use direct unaligned writes
            // instead of manual byte manipulation + SIMD operations

            // Write flags (2 bytes) directly as a single u16
            let flags_be = (*header.flags).to_be();
            std::ptr::write_unaligned(buf_ptr.add(1) as *mut u16, flags_be);

            // Write payload length (4 bytes) directly as a single u32
            let payload_be = header.payload_len.to_be();
            std::ptr::write_unaligned(buf_ptr.add(3) as *mut u32, payload_be);

            // Write sequence number (8 bytes) directly as a single u64
            let seq_be = header.sequence_number.to_be();
            std::ptr::write_unaligned(buf_ptr.add(7) as *mut u64, seq_be);

            buffer.assume_init()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::tests::{test_deserializer, test_serializer};

    #[test]
    fn test_standard_serializer() {
        test_serializer::<StandardHeaderParser>()
    }

    #[test]
    fn test_standard_deserializer() {
        test_deserializer::<StandardHeaderParser>()
    }
}
