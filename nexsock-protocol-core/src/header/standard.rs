use crate::{
    message_flags::MessageFlags,
    header::Header,
    constants::HEADER_SIZE,
    traits::header::HeaderParser
};

pub struct StandardHeaderParser;

impl HeaderParser for StandardHeaderParser {
    fn parse(bytes: &[u8]) -> Option<Header> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }
        
        let id_version = bytes[0];
        let id = (id_version & Header::LAST_SIX_BITS) >> 2;
        let version = id_version & Header::LAST_TWO_BITS;
        
        let flags = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
        
        let payload_len = ((bytes[3] as u32) << 24) |
            ((bytes[4] as u32) << 16) |
            ((bytes[5] as u32) << 8) |
            (bytes[6] as u32);
        
        let sequence_number = ((bytes[7] as u64) << 56) |
            ((bytes[8] as u64) << 48) |
            ((bytes[9] as u64) << 40) |
            ((bytes[10] as u64) << 32) |
            ((bytes[11] as u64) << 24) |
            ((bytes[12] as u64) << 16) |
            ((bytes[13] as u64) << 8) |
            (bytes[14] as u64);

        Some(Header::new(
            id,
            version,
            MessageFlags::from(flags),
            payload_len,
            sequence_number
        ))
    }
}
