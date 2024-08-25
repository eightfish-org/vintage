mod data;
mod inbound_service;
mod outbound_service;

use self::data::*;
pub use self::inbound_service::*;
pub use self::outbound_service::*;

use serde::{Deserialize, Serialize};
use vintage_msg::{BlockChainApi, ProxyMsgChannels};
use vintage_utils::ServiceStarter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub redis_addr: String,
}

pub enum Proxy {}

impl Proxy {
    pub async fn create<TApi>(
        config: ProxyConfig,
        channels: ProxyMsgChannels,
        blockchain_api: TApi,
    ) -> anyhow::Result<(
        ServiceStarter<ProxyInboundService<TApi>>,
        ServiceStarter<ProxyOutboundService>,
    )>
    where
        TApi: BlockChainApi + Send + Sync + 'static,
    {
        log::info!("connect to redis: {}", &*config.redis_addr);
        let redis_client = redis::Client::open(&*config.redis_addr).unwrap();
        let inbound_pub_sub = redis_client.get_async_connection().await?.into_pubsub();
        let inbound_redis_conn = redis_client.get_async_connection().await?;
        let outbound_redis_conn = redis_client.get_async_connection().await?;

        let inbound_service = ServiceStarter::new_with_input(
            ProxyInboundService::new(
                inbound_redis_conn,
                channels.blockchain_msg_sender,
                blockchain_api,
            ),
            inbound_pub_sub,
        );
        let outbound_service = ServiceStarter::new(ProxyOutboundService::new(
            outbound_redis_conn,
            channels.msg_receiver,
        ));

        Ok((inbound_service, outbound_service))
    }
}
