mod app;
mod test;

use crate::app::Vintage;
use crate::test::start_vintage_test;
use log::LevelFilter;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;
use vintage_msg::msg_channels;
use vintage_network::config::NodeConfig;

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

    let (
        worker_msg_sender,
        blockchain_msg_sender,
        consensus_msg_sender,
        network_msg_sender,
        worker_chn,
        state_chn,
        blockchain_chn,
        consensus_chn,
        network_chn,
    ) = msg_channels();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] != "-c" {
        print_usage();
        process::exit(1);
    }

    let config_file = &args[2];
    let config = load_config(config_file)?;

    start_vintage_test(
        worker_msg_sender,
        blockchain_msg_sender,
        consensus_msg_sender,
        network_msg_sender,
    );

    let app = Vintage::create(
        worker_chn,
        state_chn,
        blockchain_chn,
        consensus_chn,
        network_chn,
        config,
    )
    .await?;
    app.start_service().await?;

    Ok(())
}
