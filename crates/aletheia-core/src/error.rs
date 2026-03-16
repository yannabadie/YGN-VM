use thiserror::Error;

#[derive(Debug, Error)]
pub enum AletheiaError {
    #[error("hash chain broken at receipt {index}: expected {expected}, got {actual}")]
    BrokenChain {
        index: u64,
        expected: String,
        actual: String,
    },

    #[error("merkle root mismatch: expected {expected}, got {actual}")]
    MerkleRootMismatch { expected: String, actual: String },

    #[error("chain head mismatch: expected {expected}, got {actual}")]
    ChainHeadMismatch { expected: String, actual: String },

    #[error("event hash mismatch at receipt {index}: expected {expected}, got {actual}")]
    EventHashMismatch {
        index: u64,
        expected: String,
        actual: String,
    },

    #[error("invalid signature from signer {signer}")]
    InvalidSignature { signer: String },

    #[error("signing error: {0}")]
    SigningError(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("empty evidence pack")]
    EmptyPack,
}

pub type Result<T> = std::result::Result<T, AletheiaError>;
