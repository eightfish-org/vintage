mod app;
mod node;
mod node_dev;
mod test;

use crate::app::Vintage;
use crate::node::VintageNode;
use crate::node_dev::VintageNodeDev;
use log::LevelFilter;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;
use vintage_msg::msg_channels;
use vintage_network::config::NodeConfig;
use vintage_utils::start_service;

fn print_usage() {
    println!("Usage: exe -c [config_path]]");
    println!("  <config_path>: the configuration file path");
}

fn load_config(file_path: &str) -> Result<NodeConfig, anyhow::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: NodeConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // args
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "-c" {
        print_usage();
        process::exit(1);
    }
    // config
    let config_file = &args[2];
    let config = load_config(config_file)?;

    // channels
    let (
        proxy_msg_sender,
        blockchain_msg_sender,
        consensus_msg_sender,
        network_msg_sender,
        proxy_chn,
        blockchain_chn,
        consensus_chn,
        network_chn,
    ) = msg_channels();

    // vintage
    let (vintage, blockchain, blockchain_api) = Vintage::create(
        proxy_chn,
        blockchain_chn,
        config.db_path.clone(),
        config.redis_addr.clone(),
    )
    .await?;
    let join_vintage = vintage.start_service();

    // node
    let join_node = if config.dev_mode {
        let node = VintageNodeDev::create(blockchain, blockchain_api).await?;
        start_service(node, ())
    } else {
        let node = VintageNode::create(
            config,
            network_chn,
            consensus_msg_sender,
            network_msg_sender,
            consensus_chn.msg_receiver,
            blockchain,
            blockchain_api,
        )
        .await?;
        start_service(node, ())
    };

    join_vintage.await?;
    join_node.await?;

    Ok(())
}
