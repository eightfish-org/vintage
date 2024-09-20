mod constants;
mod io_object;
mod service_admin2vin;
mod service_gate2vin;
mod service_vin2worker;

use self::constants::*;
use self::io_object::*;
pub use self::service_admin2vin::*;
pub use self::service_gate2vin::*;
pub use self::service_vin2worker::*;

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
        ServiceStarter<Vin2Worker>,
        ServiceStarter<Gate2Vin<TApi>>,
        ServiceStarter<Admin2Vin>,
    )>
    where
        TApi: BlockChainApi + Send + Sync + 'static,
    {
        log::info!("connect to redis: {}", config.redis_addr);
        let redis_client = redis::Client::open(config.redis_addr)?;

        let vin2worker_conn = redis_client.get_async_connection().await?;
        let gate2vin_conn = redis_client.get_async_connection().await?;
        let gate2vin_pub_sub = redis_client.get_async_connection().await?.into_pubsub();
        let admin2vin_pub_sub = redis_client.get_async_connection().await?.into_pubsub();

        let vin2worker_starter =
            ServiceStarter::new(Vin2Worker::new(vin2worker_conn, channels.msg_receiver));
        let gate2vin_starter = ServiceStarter::new_with_input(
            Gate2Vin::new(
                gate2vin_conn,
                channels.blockchain_msg_sender.clone(),
                blockchain_api,
            ),
            gate2vin_pub_sub,
        );
        let admin2vin_starter = ServiceStarter::new_with_input(
            Admin2Vin::new(channels.blockchain_msg_sender),
            admin2vin_pub_sub,
        );

        Ok((vin2worker_starter, gate2vin_starter, admin2vin_starter))
    }
}
