use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use vintage_msg::{NetworkMsgHandler, NodeId};
use vintage_network::client::NetworkClient;
use vintage_utils::{BincodeDeserialize, BincodeSerialize};

pub struct NetworkClientWrapper {
    client: NetworkClient,
    active_number_of_nodes: usize,
}

impl NetworkClientWrapper {
    pub fn new(client: NetworkClient, active_number_of_nodes: usize) -> Self {
        Self {
            client,
            active_number_of_nodes,
        }
    }

    pub async fn request<TRequest, TResponse>(
        &mut self,
        node_id: NodeId,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
    ) -> anyhow::Result<TResponse>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
    {
        let req_encoded = request.bincode_serialize()?;
        let rsp_encoded = self
            .client
            .request(node_id, handler, req_encoded, timeout)
            .await?;
        let (rsp, _bytes_read) = TResponse::bincode_deserialize(&rsp_encoded)?;
        Ok(rsp)
    }

    pub async fn request_broadcast<TRequest, TResponse>(
        &mut self,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
    ) -> anyhow::Result<(Vec<NodeId>, TResponse)>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
    {
        log::info!(
            "====request_broadcast with active_number_of_nodes: {}",
            self.active_number_of_nodes
        );

        let encoded = request.bincode_serialize()?;
        let (node_ids, rsp_encoded) = self
            .client
            .request_broadcast(self.active_number_of_nodes, handler, encoded, timeout)
            .await?;
        let (rsp, _bytes_read) = TResponse::bincode_deserialize(&rsp_encoded)?;
        Ok((node_ids, rsp))
    }

    pub async fn request_broadcast_1<TRequest, TResponse>(
        &mut self,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
    ) -> anyhow::Result<(NodeId, TResponse)>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
    {
        let (node_ids, data) = self.request_broadcast(handler, request, timeout).await?;
        Ok((node_ids.first().unwrap().clone(), data))
    }
}
