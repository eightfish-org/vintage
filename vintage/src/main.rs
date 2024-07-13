mod app;
mod test;

use crate::app::Vintage;
use crate::test::start_vintage_test;
use log::LevelFilter;
use vintage_msg::msg_channels;

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
        blockchain_chn,
        consensus_chn,
        network_chn,
    ) = msg_channels();

    start_vintage_test(
        worker_msg_sender,
        blockchain_msg_sender,
        consensus_msg_sender,
        network_msg_sender,
    );

    let app = Vintage::create(worker_chn, blockchain_chn, consensus_chn, network_chn)?;
    app.start_service().await?;

    Ok(())
}
