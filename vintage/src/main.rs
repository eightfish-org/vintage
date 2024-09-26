mod app;
mod config;
mod node;
mod test;

use crate::app::Vintage;
use crate::config::load_config;
use crate::node::{VintageNode, VintageNodeDev};
use crate::test::start_vintage_test;
use log::LevelFilter;
use std::env;
use std::process;
use std::sync::Arc;
use vintage_msg::msg_channels;
use vintage_network::client::NetworkClient;
use vintage_network::request::NetworkRequestMgr;

fn print_usage() {
    println!("Usage: exe -c [config_path]]");
    println!("  <config_path>: the configuration file path");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // if cfg!(debug_assertions) {
    //     unsafe {
    //         env::set_var("RUST_BACKTRACE", "full");
    //     }
    // }

    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        // .filter_module("overlord", LevelFilter::Warn)
        // .filter_module("vintage_network", LevelFilter::Warn)
        // .filter_module("vintage_consensus", LevelFilter::Warn)
        // .filter_module("vintage_proxy", LevelFilter::Warn)
        .init();

    // args
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "-c" {
        print_usage();
        process::exit(1);
    }

    // config
    let config = load_config(&args[2])?;

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

    // test
    if config.dev_mode {
        start_vintage_test(&config.node.name, blockchain_msg_sender);
    }

    // network client
    let request_mgr = Arc::new(std::sync::Mutex::new(NetworkRequestMgr::new()));
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
    let join_node = if config.dev_mode {
        VintageNodeDev::create(config.node.block_interval, block_consensus)
            .await?
            .start()
    } else {
        VintageNode::create(
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
