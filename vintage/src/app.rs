use tokio::task::JoinHandle;
use vintage_blockchain::{BlockChain, BlockChainApiImpl, BlockChainConfig, BlockChainService};
use vintage_msg::{BlockChainMsgChannels, ProxyMsgChannels};
use vintage_proxy::{Proxy, ProxyConfig};
use vintage_utils::start_service;

#[allow(dead_code)]
pub struct Vintage {
    tx_service: BlockChainService,
    proxy: Proxy<BlockChainApiImpl>,
}

impl Vintage {
    pub async fn create(
        blockchain_config: BlockChainConfig,
        proxy_config: ProxyConfig,
        blockchain_chn: BlockChainMsgChannels,
        proxy_chn: ProxyMsgChannels,
    ) -> anyhow::Result<(Self, BlockChain)> {
        let (blockchain, blockchain_api, tx_service) =
            BlockChain::create(blockchain_config, blockchain_chn).await?;
        let proxy = Proxy::create(proxy_config, proxy_chn, blockchain_api).await?;

        Ok((Self { tx_service, proxy }, blockchain))
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
