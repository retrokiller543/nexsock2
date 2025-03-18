use bincode::error::DecodeError;
use thiserror::Error;

pub type ProtocolResult<T, E = ProtocolError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
