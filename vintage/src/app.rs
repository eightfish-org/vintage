use tokio::task::JoinHandle;
use vintage_blockchain::{
    BlockChain, BlockChainApiImpl, BlockChainConfig, BlockChainService, BlockConsensusImpl,
    BlockSyncService,
};
use vintage_msg::{BlockChainMsgChannels, ProxyMsgChannels};
use vintage_network::client::NetworkClient;
use vintage_proxy::{Admin2Vin, Gate2Vin, Proxy, ProxyConfig, Vin2Worker};
use vintage_utils::ServiceStarter;

#[allow(dead_code)]
pub struct Vintage {
    blockchain_service: ServiceStarter<BlockChainService>,
    block_sync_service: ServiceStarter<BlockSyncService>,
    vin_2_worker_service: ServiceStarter<Vin2Worker>,
    gate_2_vin_service: ServiceStarter<Gate2Vin<BlockChainApiImpl>>,
    admin_2_vin_service: ServiceStarter<Admin2Vin>,
}

impl Vintage {
    pub async fn create(
        blockchain_config: BlockChainConfig,
        proxy_config: ProxyConfig,
        block_interval: u64,
        active_number_of_nodes: usize,
        blockchain_chn: BlockChainMsgChannels,
        proxy_chn: ProxyMsgChannels,
        client: NetworkClient,
    ) -> anyhow::Result<(Self, BlockConsensusImpl)> {
        let (block_consensus, blockchain_api, blockchain_service, block_sync_service) =
            BlockChain::create(
                blockchain_config,
                block_interval,
                active_number_of_nodes,
                blockchain_chn,
                client,
            )
            .await?;
        let (vin_2_worker_service, gate_2_vin_service, admin_2_vin_service) =
            Proxy::create(proxy_config, proxy_chn, blockchain_api).await?;

        Ok((
            Self {
                blockchain_service,
                block_sync_service,
                vin_2_worker_service,
                gate_2_vin_service,
                admin_2_vin_service,
            },
            block_consensus,
        ))
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let join_blockchain_service = self.blockchain_service.start();
        let join_block_sync_service = self.block_sync_service.start();
        let join_gate_2_vin_service = self.gate_2_vin_service.start();
        let join_vin_2_worker_service = self.vin_2_worker_service.start();
        let join_admin_2_vin_service = self.admin_2_vin_service.start();

        tokio::spawn(async {
            let _ = join_blockchain_service.await;
            let _ = join_block_sync_service.await;
            let _ = join_gate_2_vin_service.await;
            let _ = join_vin_2_worker_service.await;
            let _ = join_admin_2_vin_service.await;
        })
    }
}
