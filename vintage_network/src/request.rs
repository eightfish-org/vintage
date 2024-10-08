use crate::client::{DynNetworkResponse, DynNetworkResponseReader};
use crate::response::{NetworkResponseSimple, NetworkResponseWithFilter, NetworkResponseWithVote};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use vintage_msg::{NetworkRequestId, NodeId};

pub type ArcNetworkRequestMgr = Arc<Mutex<NetworkRequestMgr>>;

pub struct NetworkRequestMgr {
    last_request_id: NetworkRequestId,
    requests: HashMap<NetworkRequestId, DynNetworkResponse>,
}

impl NetworkRequestMgr {
    pub fn new(node_id: u16) -> Self {
        Self {
            last_request_id: node_id as u64 * 1_000_000_000_000_000_000,
            requests: HashMap::new(),
        }
    }

    pub(super) fn request(&mut self) -> (NetworkRequestId, DynNetworkResponseReader) {
        let response = Arc::new(NetworkResponseSimple::new());
        let request_id = self.insert_request(response.clone());
        (request_id, response)
    }

    pub(super) fn request_with_filter<TFilter>(
        &mut self,
        filter: TFilter,
    ) -> (NetworkRequestId, DynNetworkResponseReader)
    where
        TFilter: Fn(&[u8]) -> bool + Send + Sync + 'static,
    {
        let response = Arc::new(NetworkResponseWithFilter::new(filter));
        let request_id = self.insert_request(response.clone());
        (request_id, response)
    }

    pub(super) fn request_with_vote(
        &mut self,
        node_count: usize,
    ) -> (NetworkRequestId, DynNetworkResponseReader) {
        let response = Arc::new(NetworkResponseWithVote::new(node_count));
        let request_id = self.insert_request(response.clone());
        (request_id, response)
    }

    pub(super) fn on_response(&self, node_id: NodeId, request_id: NetworkRequestId, data: Vec<u8>) {
        if let Some(response) = self.requests.get(&request_id) {
            response.write_data(node_id, data);
        } else {
            log::debug!(
                "request {} removed, received response from {}",
                request_id,
                node_id
            );
        }
    }

    pub(super) fn remove(&mut self, request_id: NetworkRequestId) {
        self.requests.remove(&request_id);
    }

    fn insert_request(&mut self, response: DynNetworkResponse) -> NetworkRequestId {
        self.last_request_id += 1;
        self.requests.insert(self.last_request_id, response);
        self.last_request_id
    }
}
