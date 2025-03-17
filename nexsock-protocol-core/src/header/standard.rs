use crate::{
    message_flags::MessageFlags,
    header::Header,
    constants::HEADER_SIZE,
    traits::header::HeaderDeserializer
};
use crate::traits::header::HeaderSerializer;

pub struct StandardHeaderParser;

impl HeaderDeserializer for StandardHeaderParser {
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

impl HeaderSerializer for StandardHeaderParser {
    fn serialize(header: &Header) -> [u8; HEADER_SIZE] {
        let mut buffer = [0; HEADER_SIZE];
        buffer[0] = ((header.id & Header::LAST_SIX_BITS) << 2) | (header.version & Header::LAST_TWO_BITS);

        buffer[1] = (*header.flags >> 8) as u8;
        buffer[2] = *header.flags as u8;

        buffer[3] = (header.payload_len >> 24) as u8;
        buffer[4] = (header.payload_len >> 16) as u8;
        buffer[5] = (header.payload_len >> 8) as u8;
        buffer[6] = header.payload_len as u8;

        buffer[7] = (header.sequence_number >> 56) as u8;
        buffer[8] = (header.sequence_number >> 48) as u8;
        buffer[9] = (header.sequence_number >> 40) as u8;
        buffer[10] = (header.sequence_number >> 32) as u8;
        buffer[11] = (header.sequence_number >> 24) as u8;
        buffer[12] = (header.sequence_number >> 16) as u8;
        buffer[13] = (header.sequence_number >> 8) as u8;
        buffer[14] = header.sequence_number as u8;
        
        buffer
    }
}
