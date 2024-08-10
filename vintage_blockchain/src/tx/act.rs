use sha2::{Digest, Sha256};
use vintage_msg::{Act, Hashed};

pub type ActId = Hashed;

pub(crate) fn calc_act_id(act: &Act) -> ActId {
    let mut hasher = Sha256::new();
    hasher.update(act.kind.as_bytes());
    hasher.update(act.model.as_bytes());
    hasher.update(&act.data);
    ActId::new(hasher.finalize().into())
}
