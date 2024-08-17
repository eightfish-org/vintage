use std::sync::Arc;
use tokio::task::JoinHandle;
use vintage_blockchain::{BlockChain, BlockChainApiImpl, TxService};
use vintage_msg::{BlockChainMsgChannels, ProxyMsgChannels};
use vintage_proxy::Proxy;
use vintage_utils::start_service;

#[allow(dead_code)]
pub struct Vintage {
    tx_service: TxService,
    proxy: Proxy<BlockChainApiImpl>,
}

impl Vintage {
    pub async fn create(
        proxy_chn: ProxyMsgChannels,
        blockchain_chn: BlockChainMsgChannels,
        db_path: String,
        redis_addr: String,
    ) -> anyhow::Result<(Self, BlockChain, Arc<BlockChainApiImpl>)> {
        log::info!("Vintage config db path: {}", db_path);
        let (blockchain, blockchain_api, tx_service) =
            BlockChain::create(blockchain_chn, db_path).await?;
        let proxy = Proxy::create(proxy_chn, blockchain_api.clone(), redis_addr.as_str()).await?;

        Ok((Self { tx_service, proxy }, blockchain, blockchain_api))
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let tx_service = start_service(self.tx_service, ());
        let proxy_inbound = start_service(self.proxy.inbound, self.proxy.pub_sub);
        let proxy_outbound = start_service(self.proxy.outbound, ());

        tokio::spawn(async {
            let _ = tx_service.await;
            let _ = proxy_inbound.await;
            let _ = proxy_outbound.await;
        })
    }
}
