mod data;
mod inbound_service;
mod outbound_service;

use crate::inbound_service::ProxyInboundService;
use crate::outbound_service::ProxyOutboundService;
use redis::aio::PubSub;
use vintage_msg::ProxyMsgChannels;

const SUBNODE_RPC_ENV: &str = "SUBNODE_RPC";
const REDIS_ADDRESS_ENV: &str = "REDIS_URL";

pub struct Proxy<TApi> {
    pub inbound: ProxyInboundService<TApi>,
    pub outbound: ProxyOutboundService,
    pub pub_sub: PubSub,
}

impl<TApi> Proxy<TApi> {
    pub async fn create(channels: ProxyMsgChannels, blockchain_api: TApi) -> anyhow::Result<Self> {
        let rpc_addr = std::env::var(SUBNODE_RPC_ENV)?;
        println!("rpc_addr: {}", rpc_addr);
        let redis_addr = std::env::var(REDIS_ADDRESS_ENV)?;
        println!("redis_addr: {}", redis_addr);
        let redis_client = redis::Client::open(redis_addr).unwrap();
        let redis_conn = redis_client.get_async_connection().await?;
        let redis_conn_2 = redis_client.get_async_connection().await?;
        let pub_sub = redis_client.get_async_connection().await?.into_pubsub();

        let inbound =
            ProxyInboundService::new(redis_conn_2, channels.blockchain_msg_sender, blockchain_api);
        let outbound = ProxyOutboundService::new(redis_conn, channels.msg_receiver);

        Ok(Self {
            inbound,
            outbound,
            pub_sub,
        })
    }
}
