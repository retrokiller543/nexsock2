use crate::constants::HEADER_SIZE;
use crate::header::Header;
use crate::message_flags::MessageFlags;
use crate::traits::header::HeaderDeserializer;

pub struct OptimizedHeaderParser;

impl HeaderDeserializer for OptimizedHeaderParser {
    #[inline]
    fn parse(buf: &[u8]) -> Option<Header> {
        if buf.len() < HEADER_SIZE {
            return None;
        }

        unsafe {
            let header_bytes = std::ptr::read_unaligned(buf.as_ptr() as *const [u8; HEADER_SIZE]);
            
            let first_byte = header_bytes[0];
            let id = (first_byte & Header::LAST_SIX_BITS) >> 2;
            let version = first_byte & Header::LAST_TWO_BITS;
            
            let flags = u16::from_be_bytes([header_bytes[1], header_bytes[2]]);
            
            let payload_len = u32::from_be_bytes([
                header_bytes[3], header_bytes[4], header_bytes[5], header_bytes[6]
            ]);
            
            let sequence_number = u64::from_be_bytes([
                header_bytes[7], header_bytes[8], header_bytes[9], header_bytes[10],
                header_bytes[11], header_bytes[12], header_bytes[13], header_bytes[14]
            ]);

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
    use crate::header::tests::test_deserializer;

    #[test]
    fn test_optimized_deserializer() {
        test_deserializer::<OptimizedHeaderParser>()
    }
}
