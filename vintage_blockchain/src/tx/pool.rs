use std::mem;
use vintage_msg::Tx;

pub(crate) struct TxPool<const CAPACITY: usize> {
    vec: Vec<Tx>,
}

impl<const CAPACITY: usize> TxPool<CAPACITY> {
    pub fn new() -> Self {
        Self {
            vec: Self::new_tx_vec(),
        }
    }

    pub fn take_tx_vec(&mut self) -> Vec<Tx> {
        mem::replace(&mut self.vec, Self::new_tx_vec())
    }

    fn new_tx_vec() -> Vec<Tx> {
        Vec::with_capacity(CAPACITY)
    }
}
