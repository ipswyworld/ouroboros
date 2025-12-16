pub mod microchain;
pub mod transaction;
pub mod client;
pub mod types;
pub mod error;

pub use microchain::Microchain;
pub use transaction::Transaction;
pub use client::OuroClient;
pub use types::{MicrochainConfig, ConsensusType, AnchorFrequency};
pub use error::{SdkError, Result};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::microchain::Microchain;
    pub use crate::transaction::Transaction;
    pub use crate::client::OuroClient;
    pub use crate::types::*;
    pub use crate::error::{SdkError, Result};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
