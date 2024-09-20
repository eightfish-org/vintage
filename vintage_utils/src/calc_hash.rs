use crate::Hashed;
use digest::Digest;
use sha2::Sha256;

pub trait CalcHash {
    fn calc_hash(&self) -> Hashed;
}

impl CalcHash for [u8] {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(self);
        hasher.into()
    }
}

impl CalcHash for u64 {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(self.to_be_bytes());
        hasher.into()
    }
}
