use anyhow::anyhow;
use async_trait::async_trait;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;
use vintage_msg::NodeId;
use vintage_utils::{Activation, Data};

pub trait NetworkResponseIO: NetworkResponseWriter + NetworkResponseReader {}

pub trait NetworkResponseWriter {
    fn write_data(&self, node_id: NodeId, data: Vec<u8>);
}

#[async_trait]
pub trait NetworkResponseReader: Send + Sync {
    async fn read_data(&self, timeout: Duration) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)>;
}

pub struct NetworkSingleResponse {
    data: Data<Vec<u8>>,
}

impl NetworkSingleResponse {
    pub fn new() -> Self {
        Self {
            data: Data::new(Default::default()),
        }
    }
}

impl NetworkResponseIO for NetworkSingleResponse {}

impl NetworkResponseWriter for NetworkSingleResponse {
    fn write_data(&self, _node_id: NodeId, data: Vec<u8>) {
        self.data.set_data(data);
    }
}

#[async_trait]
impl NetworkResponseReader for NetworkSingleResponse {
    async fn read_data(&self, timeout: Duration) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)> {
        let data = tokio::time::timeout(timeout, self.data.clone_data()).await?;
        Ok((Default::default(), data))
    }
}

pub struct NetworkMultiResponse {
    activation: Activation,
    node_count: usize,
    multi_data: Mutex<HashMap<Vec<u8>, HashSet<NodeId>>>,
}

impl NetworkMultiResponse {
    pub fn new(node_count: usize) -> Self {
        Self {
            activation: Activation::new(false),
            node_count,
            multi_data: Mutex::new(HashMap::with_capacity(1)),
        }
    }
}

impl NetworkResponseIO for NetworkMultiResponse {}

impl NetworkResponseWriter for NetworkMultiResponse {
    fn write_data(&self, node_id: NodeId, data: Vec<u8>) {
        let mut guard = self.multi_data.lock().unwrap();
        match guard.entry(data) {
            Entry::Occupied(mut entry) => {
                let node_ids = entry.get_mut();
                node_ids.insert(node_id);
                if node_ids.len() >= self.node_count {
                    self.activation.set_active(true);
                }
            }
            Entry::Vacant(entry) => {
                let mut node_ids = HashSet::with_capacity(self.node_count);
                node_ids.insert(node_id);
                entry.insert(node_ids);
            }
        }
    }
}

#[async_trait]
impl NetworkResponseReader for NetworkMultiResponse {
    async fn read_data(&self, timeout: Duration) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)> {
        tokio::time::timeout(timeout, self.activation.wait()).await?;
        {
            let guard = self.multi_data.lock().unwrap();
            for (data, node_ids) in &*guard {
                if node_ids.len() >= self.node_count {
                    let node_ids: Vec<NodeId> = node_ids.iter().cloned().collect();
                    return Ok((node_ids, data.clone()));
                }
            }
        }
        Err(anyhow!("network multi response err"))
    }
}
