mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    Ok(())
}
