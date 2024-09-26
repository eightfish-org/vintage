mod app;
mod args;
mod config;
mod dev;
mod logger;
mod node;

use crate::app::Vintage;
use crate::args::args;
use crate::config::{load_config, VintageMode};
use crate::dev::start_dev_task;
use crate::logger::env_logger_init;
use crate::node::{VintageMultiNode, VintageSingleNode};
use std::sync::Arc;
use vintage_msg::msg_channels;
use vintage_network::client::NetworkClient;
use vintage_network::request::NetworkRequestMgr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // if cfg!(debug_assertions) {
    //     unsafe {
    //         env::set_var("RUST_BACKTRACE", "full");
    //     }
    // }

    // env_logger
    env_logger_init();

    // args
    let config_file = args();

    // config
    let config = load_config(&config_file)?;

    // channels
    #[allow(unused_variables)]
    let (
        blockchain_msg_sender,
        proxy_msg_sender,
        consensus_msg_sender,
        network_msg_sender,
        blockchain_chn,
        proxy_chn,
        consensus_chn,
        network_chn,
    ) = msg_channels();

    // dev
    if config.mode != VintageMode::Prod {
        start_dev_task(&config.node.name, blockchain_msg_sender);
    }

    // network client
    let request_mgr = Arc::new(std::sync::Mutex::new(NetworkRequestMgr::new(
        config.node.id,
    )));
    let client = NetworkClient::new(request_mgr.clone(), network_msg_sender.clone());

    // vintage
    let (vintage, block_consensus) = Vintage::create(
        config.blockchain,
        config.proxy,
        config.node.block_interval,
        config.node.get_number_of_node() * 2 / 3,
        blockchain_chn,
        proxy_chn,
        client,
    )
    .await?;
    let join_vintage = vintage.start_service();

    // node
    let join_node = if config.mode == VintageMode::DevSingleNode {
        VintageSingleNode::create(config.node.block_interval, block_consensus)
            .await?
            .start()
    } else {
        VintageMultiNode::create(
            config.node,
            consensus_chn,
            network_chn,
            block_consensus,
            request_mgr,
        )
        .await?
        .start()
    };

    join_vintage.await?;
    join_node.await?;
    Ok(())
}
