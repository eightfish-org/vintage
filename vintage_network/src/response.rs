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

////////////////////////////////////////////////////////////////////////////////////////////////////
// NetworkResponseSimple

pub struct NetworkResponseSimple {
    node_id_and_data: Data<Option<(NodeId, Vec<u8>)>>,
}

impl NetworkResponseSimple {
    pub fn new() -> Self {
        Self {
            node_id_and_data: Data::new(None),
        }
    }
}

impl NetworkResponseIO for NetworkResponseSimple {}

impl NetworkResponseWriter for NetworkResponseSimple {
    fn write_data(&self, node_id: NodeId, data: Vec<u8>) {
        self.node_id_and_data.set_data(Some((node_id, data)));
    }
}

#[async_trait]
impl NetworkResponseReader for NetworkResponseSimple {
    async fn read_data(&self, timeout: Duration) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)> {
        let (node_id, data) = tokio::time::timeout(timeout, self.node_id_and_data.clone_data())
            .await?
            .unwrap();
        Ok((vec![node_id], data))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NetworkResponseWithFilter

pub struct NetworkResponseWithFilter<TFilter> {
    node_id_and_data: Data<Option<(NodeId, Vec<u8>)>>,
    filter: TFilter,
}

impl<TFilter> NetworkResponseWithFilter<TFilter> {
    pub fn new(filter: TFilter) -> Self {
        Self {
            node_id_and_data: Data::new(None),
            filter,
        }
    }
}

impl<TFilter> NetworkResponseIO for NetworkResponseWithFilter<TFilter> where
    TFilter: Fn(&[u8]) -> bool + Send + Sync
{
}

impl<TFilter> NetworkResponseWriter for NetworkResponseWithFilter<TFilter>
where
    TFilter: Fn(&[u8]) -> bool,
{
    fn write_data(&self, node_id: NodeId, data: Vec<u8>) {
        if (self.filter)(&data) {
            self.node_id_and_data.set_data(Some((node_id, data)));
        }
    }
}

#[async_trait]
impl<TFilter> NetworkResponseReader for NetworkResponseWithFilter<TFilter>
where
    TFilter: Send + Sync,
{
    async fn read_data(&self, timeout: Duration) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)> {
        let (node_id, data) = tokio::time::timeout(timeout, self.node_id_and_data.clone_data())
            .await?
            .unwrap();
        Ok((vec![node_id], data))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NetworkResponseWithVote

pub struct NetworkResponseWithVote {
    activation: Activation,
    multi_data: Mutex<HashMap<Vec<u8>, HashSet<NodeId>>>,
    node_count: usize,
}

impl NetworkResponseWithVote {
    pub fn new(node_count: usize) -> Self {
        Self {
            activation: Activation::new(false),
            multi_data: Mutex::new(HashMap::with_capacity(1)),
            node_count,
        }
    }
}

impl NetworkResponseIO for NetworkResponseWithVote {}

impl NetworkResponseWriter for NetworkResponseWithVote {
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
impl NetworkResponseReader for NetworkResponseWithVote {
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
        Err(anyhow!("NetworkResponseWithVote read_data err"))
    }
}
