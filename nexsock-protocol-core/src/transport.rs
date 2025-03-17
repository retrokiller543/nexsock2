use std::io;
use bytes::{Bytes, BytesMut};
use futures::{AsyncRead, AsyncReadExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use crate::constants::HEADER_SIZE;
use crate::error::ProtocolResult;
use crate::frame::Frame;
use crate::header::Header;
use crate::message_flags::MessageFlags;
use crate::traits::MessageBody;

pub struct Transport<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> {
    reader: R,
    writer: W,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Transport<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    pub async fn read_message(&mut self) -> ProtocolResult<impl MessageBody> {
        self.read_magic().await?;

        let mut buf = BytesMut::with_capacity(HEADER_SIZE);

        self.reader.read_exact(&mut buf).await?;

        let header = Header::parse::<crate::header::standard::StandardHeaderParser>(&mut buf.freeze()).unwrap();

        if header.flags().contains(MessageFlags::HAS_PAYLOAD) && header.payload_len() > 0 {
            self.read_body(header).await
        } else {
            Ok(())
        }
    }

    async fn read_body<T: MessageBody>(&mut self, header: Header) -> ProtocolResult<T> {
        let payload_len = header.payload_len();

        let mut buffer = BytesMut::with_capacity(payload_len as usize);

        self.reader.read_exact(&mut buffer).await?;

        let bytes = buffer.freeze();
        let config = bincode::config::standard().with_big_endian();
        
        bincode::decode_from_slice(&bytes, config).map_err(Into::into).map(|(data, _)| data)
    }

    async fn read_magic(&mut self) -> ProtocolResult<()> {
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic).await?;

        if &magic != b"NEX\0" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid protocol magic bytes",
            ).into());
        }

        Ok(())
    }

    pub async fn write_message<T: MessageBody>(&mut self, message: Frame<{ HEADER_SIZE }, T>) {
        let mut buf = [0u8; 1024];
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use futures::task::noop_waker;
    use std::io::{Error, ErrorKind};
    use bincode::{Decode, Encode};
    use bytes::BufMut;
    use tokio::io::ReadBuf;

    // Mock structures for testing
    pub(crate) struct MockReader {
        data: Vec<u8>,
        position: usize,
    }

    impl MockReader {
        pub(crate) fn new(data: Vec<u8>) -> Self {
            Self {
                data,
                position: 0,
            }
        }
    }

    impl AsyncRead for MockReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            if self.position >= self.data.len() {
                return Poll::Ready(Ok(0));
            }

            let available = self.data.len() - self.position;
            let read_len = std::cmp::min(buf.len(), available);

            buf[0..read_len].copy_from_slice(&self.data[self.position..self.position + read_len]);
            self.position += read_len;

            Poll::Ready(Ok(read_len))
        }
    }

    struct MockWriter {
        data: Vec<u8>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                data: Vec::new(),
            }
        }

        fn written_data(&self) -> &[u8] {
            &self.data
        }
    }

    impl AsyncWrite for MockWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.data.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    // Define a simple message body for testing
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct TestMessage {
        field1: u32,
        field2: String,
    }

    impl MessageBody for TestMessage {}
/*
    #[tokio::test]
    async fn test_read_message() {
        // Create test data
        let mut test_data = Vec::new();

        // Add magic bytes
        test_data.extend_from_slice(b"NEX\0");

        // Create a header
        let id = 5;
        let version = 1;
        let flags = MessageFlags::HAS_PAYLOAD;
        let test_message = TestMessage {
            field1: 42,
            field2: "Hello, world!".to_string(),
        };

        // Serialize the message
        let config = bincode::config::standard().with_big_endian();
        let payload = bincode::encode_to_vec(&test_message, config).unwrap();
        let payload_len = payload.len() as u32;
        let sequence_number = 123;

        // Add header bytes
        let header = Header::new(id, version, flags, payload_len, sequence_number);
        let header_bytes = header.to_bytes();
        test_data.extend_from_slice(&header_bytes);

        // Add payload
        test_data.extend_from_slice(&payload);

        // Create mock reader and writer
        let reader = MockReader::new(test_data);
        let writer = MockWriter::new();

        // Create transport
        let mut transport = Transport::new(reader, writer);

        // Read message
        let result: TestMessage = transport.read_message().await.unwrap();

        // Verify result
        assert_eq!(result.field1, 42);
        assert_eq!(result.field2, "Hello, world!");
    }

    #[tokio::test]
    async fn test_read_message_invalid_magic() {
        // Create test data with invalid magic
        let mut test_data = Vec::new();

        // Add invalid magic bytes
        test_data.extend_from_slice(b"INVALID");

        // Create mock reader and writer
        let reader = MockReader::new(test_data);
        let writer = MockWriter::new();

        // Create transport
        let mut transport = Transport::new(reader, writer);

        // Read message should fail with error
        let result: Result<(), _> = transport.read_message().await;
        assert!(result.is_err());

        // Verify error is about invalid magic bytes
        let err = result.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Invalid protocol magic"));
    }

    #[tokio::test]
    async fn test_read_message_no_payload() {
        // Create test data
        let mut test_data = Vec::new();

        // Add magic bytes
        test_data.extend_from_slice(b"NEX\0");

        // Create a header with no payload flag
        let id = 5;
        let version = 1;
        let flags = MessageFlags::NONE; // No HAS_PAYLOAD flag
        let payload_len = 0;
        let sequence_number = 123;

        // Add header bytes
        let header = Header::new(id, version, flags, payload_len, sequence_number);
        let header_bytes = header.to_bytes();
        test_data.extend_from_slice(&header_bytes);

        // Create mock reader and writer
        let reader = MockReader::new(test_data);
        let writer = MockWriter::new();

        // Create transport
        let mut transport = Transport::new(reader, writer);

        // Read message
        let result: () = transport.read_message().await.unwrap();

        // Verify result is unit type (no payload)
        assert_eq!(result, ());
    }*/
}