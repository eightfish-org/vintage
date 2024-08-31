use crate::request::ArcNetworkRequestMgr;
use crate::response::{NetworkResponseIO, NetworkResponseReader};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::error::Elapsed;
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

    pub async fn request(
        &self,
        timeout: Duration,
        node_id: NodeId,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
    ) -> Result<Vec<u8>, Elapsed> {
        let (request_id, response) = { self.request_mgr.lock().unwrap().request() };
        self.network_msg_sender
            .send_msg(MsgToNetwork::Request(node_id, handler, request_id, content));
        let result = response.read_data(timeout).await;
        {
            self.request_mgr.lock().unwrap().remove(request_id);
        }
        result
    }

    pub async fn request_broadcast(
        &self,
        timeout: Duration,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
        node_count: usize,
    ) -> Result<Vec<u8>, Elapsed> {
        let (request_id, response) = {
            self.request_mgr
                .lock()
                .unwrap()
                .request_broadcast(node_count)
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
