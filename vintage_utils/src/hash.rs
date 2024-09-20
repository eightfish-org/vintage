use anyhow::anyhow;
use bytes::Bytes;
use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fmt::{Display, Formatter};

pub const HASH_SIZE: usize = 32;
pub type HashBytes = [u8; HASH_SIZE];
const ZERO_HASH: HashBytes = [0; HASH_SIZE];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hashed(HashBytes);

impl Hashed {
    pub const fn zero_hash() -> Self {
        Self(ZERO_HASH)
    }

    pub fn as_bytes(&self) -> &HashBytes {
        &self.0
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

impl From<&HashBytes> for Hashed {
    fn from(value: &HashBytes) -> Self {
        Self(value.to_owned())
    }
}

impl From<Sha256> for Hashed {
    fn from(value: Sha256) -> Self {
        Self(value.finalize().into())
    }
}

impl TryFrom<&Bytes> for Hashed {
    type Error = anyhow::Error;

    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        if bytes.len() != HASH_SIZE {
            return Err(anyhow!("bytes len is {}", bytes.len()));
        }
        let mut hash = ZERO_HASH;
        hash.copy_from_slice(bytes);
        Ok(Self(hash))
    }
}

impl From<&Hashed> for Bytes {
    fn from(hash: &Hashed) -> Self {
        Self::copy_from_slice(&hash.0)
    }
}
