use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use vintage_blockchain::BlockChainConfig;
use vintage_network::config::NodeConfig;
use vintage_proxy::ProxyConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum VintageMode {
    Prod,
    Dev,
    Test,
    SingleNodeDev,
    SingleNodeTest,
}

impl VintageMode {
    pub fn multi_nodes_mode(self) -> bool {
        self == Self::Prod || self == Self::Dev || self == Self::Test
    }
    pub fn test_mode(self) -> bool {
        self == Self::Test || self == Self::SingleNodeTest
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VintageConfig {
    pub mode: VintageMode,
    pub blockchain: BlockChainConfig,
    pub proxy: ProxyConfig,
    pub node: NodeConfig,
}

pub fn load_config(file_path: &str) -> Result<VintageConfig, anyhow::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: VintageConfig = serde_yaml::from_str(&contents)?;

    log::info!("vintage config: {:?}", config);
    Ok(config)
}
