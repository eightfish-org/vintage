use anyhow::anyhow;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Hashed([u8; 32]);

impl Hashed {
    pub const fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}

impl Display for Hashed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl AsRef<[u8]> for Hashed {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for Hashed {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<&Hashed> for Bytes {
    fn from(hash: &Hashed) -> Self {
        Self::copy_from_slice(&hash.0)
    }
}

impl TryFrom<&Bytes> for Hashed {
    type Error = anyhow::Error;

    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(anyhow!("bytes len is {}", bytes.len()));
        }
        let mut hash = [0; 32];
        hash.copy_from_slice(bytes);
        Ok(Self(hash))
    }
}
