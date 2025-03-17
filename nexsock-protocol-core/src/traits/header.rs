use bytes::{Buf, Bytes, BytesMut};
use futures::{AsyncRead, AsyncReadExt};
use crate::constants::HEADER_SIZE;
use crate::error::ProtocolResult;
use crate::header::Header;

pub trait HeaderParser {
    fn parse(bytes: &[u8]) -> Option<Header>;

    fn parse_bytes(bytes: &mut Bytes) -> Option<Header> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }

        let slice = &bytes[..HEADER_SIZE];
        let header = Self::parse(slice)?;

        bytes.advance(HEADER_SIZE);

        Some(header)
    }

    async fn read_header<R: AsyncRead + Unpin>(
        reader: &mut R
    ) -> ProtocolResult<Header> {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE);
        buf.resize(HEADER_SIZE, 0);

        AsyncReadExt::read_exact(reader, &mut buf).await?;

        Self::parse(&buf)
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse header"
            ).into())
    }
}