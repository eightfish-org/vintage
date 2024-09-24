use crate::request::ArcNetworkRequestMgr;
use crate::response::{NetworkResponseIO, NetworkResponseReader};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_msg::{MsgToNetwork, NetworkMsgHandler, NodeId};
use vintage_utils::SendMsg;

pub type DynNetworkResponse = Arc<dyn NetworkResponseIO>;
pub type DynNetworkResponseReader = Arc<dyn NetworkResponseReader>;

pub struct NetworkClient {
    request_mgr: ArcNetworkRequestMgr,
    network_msg_sender: mpsc::Sender<MsgToNetwork>,
}

impl NetworkClient {
    pub fn new(
        request_mgr: ArcNetworkRequestMgr,
        network_msg_sender: mpsc::Sender<MsgToNetwork>,
    ) -> Self {
        Self {
            request_mgr,
            network_msg_sender,
        }
    }

    pub async fn request_with_single_node(
        &self,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
        timeout: Duration,
        node_id: NodeId,
    ) -> anyhow::Result<Vec<u8>> {
        let (request_id, response) = { self.request_mgr.lock().unwrap().request() };
        self.network_msg_sender
            .send_msg(MsgToNetwork::Request(node_id, handler, request_id, content));
        let result = response.read_data(timeout).await;
        {
            self.request_mgr.lock().unwrap().remove(request_id);
        }
        let (_node_ids, data) = result?;
        Ok(data)
    }

    pub async fn request_with_filter<TFilter>(
        &self,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
        timeout: Duration,
        filter: TFilter,
    ) -> anyhow::Result<(NodeId, Vec<u8>)>
    where
        TFilter: Fn(&[u8]) -> bool + Send + Sync + 'static,
    {
        let (request_id, response) =
            { self.request_mgr.lock().unwrap().request_with_filter(filter) };
        self.network_msg_sender
            .send_msg(MsgToNetwork::RequestBroadcast(handler, request_id, content));
        let result = response.read_data(timeout).await;
        {
            self.request_mgr.lock().unwrap().remove(request_id);
        }
        let (node_ids, data) = result?;
        Ok((node_ids.first().unwrap().clone(), data))
    }

    pub async fn request_with_vote(
        &self,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
        timeout: Duration,
        node_count: usize,
    ) -> anyhow::Result<(Vec<NodeId>, Vec<u8>)> {
        let (request_id, response) = {
            self.request_mgr
                .lock()
                .unwrap()
                .request_with_vote(node_count)
        };
        self.network_msg_sender
            .send_msg(MsgToNetwork::RequestBroadcast(handler, request_id, content));
        let result = response.read_data(timeout).await;
        {
            self.request_mgr.lock().unwrap().remove(request_id);
        }
        result
    }
}
