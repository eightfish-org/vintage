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

    pub async fn request_with_single_node<TRequest, TResponse>(
        &self,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
        node_id: NodeId,
    ) -> anyhow::Result<TResponse>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
    {
        let req_encoded = request.bincode_serialize()?;
        let rsp_encoded = self
            .client
            .request_with_single_node(handler, req_encoded, timeout, node_id)
            .await?;
        let (rsp, _bytes_read) = TResponse::bincode_deserialize(&rsp_encoded)?;
        Ok(rsp)
    }

    pub async fn request_with_filter<TRequest, TResponse, TFilter>(
        &self,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
        filter: TFilter,
    ) -> anyhow::Result<(NodeId, TResponse)>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
        TFilter: Fn(&[u8]) -> bool + Send + Sync + 'static,
    {
        let req_encoded = request.bincode_serialize()?;
        let (node_id, rsp_encoded) = self
            .client
            .request_with_filter(handler, req_encoded, timeout, filter)
            .await?;
        let (rsp, _bytes_read) = TResponse::bincode_deserialize(&rsp_encoded)?;
        Ok((node_id, rsp))
    }

    pub async fn request_with_vote<TRequest, TResponse>(
        &self,
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
            .request_with_vote(handler, encoded, timeout, self.active_number_of_nodes)
            .await?;
        let (rsp, _bytes_read) = TResponse::bincode_deserialize(&rsp_encoded)?;
        Ok((node_ids, rsp))
    }

    pub async fn request_with_vote_1<TRequest, TResponse>(
        &self,
        handler: NetworkMsgHandler,
        request: TRequest,
        timeout: Duration,
    ) -> anyhow::Result<(NodeId, TResponse)>
    where
        TRequest: Serialize,
        TResponse: DeserializeOwned,
    {
        let (node_ids, data) = self.request_with_vote(handler, request, timeout).await?;
        Ok((node_ids.first().unwrap().clone(), data))
    }
}
