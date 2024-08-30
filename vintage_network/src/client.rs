use crate::request::ArcNetworkRequestMgr;
use crate::response::{NetworkResponseIO, NetworkResponseReader};
use std::sync::Arc;
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

    pub fn request(
        &mut self,
        node_id: NodeId,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
    ) -> DynNetworkResponseReader {
        let (request_id, response) = self.request_mgr.lock().unwrap().request();
        self.network_msg_sender
            .send_msg(MsgToNetwork::Request(node_id, handler, request_id, content));
        response
    }

    pub fn request_broadcast(
        &mut self,
        handler: NetworkMsgHandler,
        content: Vec<u8>,
        node_count: usize,
    ) -> DynNetworkResponseReader {
        let (request_id, response) = self
            .request_mgr
            .lock()
            .unwrap()
            .request_broadcast(node_count);
        self.network_msg_sender
            .send_msg(MsgToNetwork::RequestBroadcast(handler, request_id, content));
        response
    }
}
