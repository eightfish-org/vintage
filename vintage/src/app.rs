use tokio::task::JoinHandle;
use vintage_blockchain::{
    BlockChain, BlockChainApiImpl, BlockChainConfig, BlockChainService, BlockConsensusImpl,
    BlockSyncService,
};
use vintage_msg::{BlockChainMsgChannels, ProxyMsgChannels};
use vintage_network::client::NetworkClient;
use vintage_proxy::{Proxy, ProxyConfig, ProxyInboundService, ProxyOutboundService};
use vintage_utils::ServiceStarter;

#[allow(dead_code)]
pub struct Vintage {
    blockchain_service: ServiceStarter<BlockChainService>,
    block_sync_service: ServiceStarter<BlockSyncService>,
    proxy_inbound_service: ServiceStarter<ProxyInboundService<BlockChainApiImpl>>,
    proxy_outbound_service: ServiceStarter<ProxyOutboundService>,
}

impl Vintage {
    pub async fn create(
        blockchain_config: BlockChainConfig,
        proxy_config: ProxyConfig,
        block_interval: u64,
        blockchain_chn: BlockChainMsgChannels,
        proxy_chn: ProxyMsgChannels,
        client: NetworkClient,
    ) -> anyhow::Result<(Self, BlockConsensusImpl)> {
        let (block_consensus, blockchain_api, blockchain_service, block_sync_service) =
            BlockChain::create(blockchain_config, block_interval, blockchain_chn, client).await?;
        let (proxy_inbound_service, proxy_outbound_service) =
            Proxy::create(proxy_config, proxy_chn, blockchain_api).await?;

        Ok((
            Self {
                blockchain_service,
                block_sync_service,
                proxy_inbound_service,
                proxy_outbound_service,
            },
            block_consensus,
        ))
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let join_blockchain_service = self.blockchain_service.start();
        let join_block_sync_service = self.block_sync_service.start();
        let join_proxy_inbound_service = self.proxy_inbound_service.start();
        let join_proxy_outbound_service = self.proxy_outbound_service.start();

        tokio::spawn(async {
            let _ = join_blockchain_service.await;
            let _ = join_block_sync_service.await;
            let _ = join_proxy_inbound_service.await;
            let _ = join_proxy_outbound_service.await;
        })
    }
}
