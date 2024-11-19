use crate::Proto;

pub struct UploadWasm {
    pub proto: Proto,
    pub wasm_binary: Vec<u8>,
    pub after_blocks: u64,
}
