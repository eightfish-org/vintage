mod data;
mod inbound_service;
mod outbound_service;

use crate::inbound_service::ProxyInboundService;
use crate::outbound_service::ProxyOutboundService;
use redis::aio::PubSub;
use redis::IntoConnectionInfo;
use vintage_msg::ProxyMsgChannels;

pub struct Proxy<TApi> {
    pub inbound: ProxyInboundService<TApi>,
    pub outbound: ProxyOutboundService,
    pub pub_sub: PubSub,
}

impl<TApi> Proxy<TApi> {
    pub async fn create(
        channels: ProxyMsgChannels,
        blockchain_api: TApi,
        redis_addr: impl IntoConnectionInfo,
    ) -> anyhow::Result<Self> {
        let redis_client = redis::Client::open(redis_addr).unwrap();
        let redis_conn = redis_client.get_async_connection().await?;
        let redis_conn_2 = redis_client.get_async_connection().await?;
        let pub_sub = redis_client.get_async_connection().await?.into_pubsub();

        let inbound =
            ProxyInboundService::new(redis_conn, channels.blockchain_msg_sender, blockchain_api);
        let outbound = ProxyOutboundService::new(redis_conn_2, channels.msg_receiver);

        Ok(Self {
            inbound,
            outbound,
            pub_sub,
        })
    }
}
