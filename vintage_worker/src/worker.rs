use tokio::task::JoinHandle;
use vintage_msg::WorkerMsgChannels;

pub struct Worker {}

impl Worker {
    #[allow(unused_variables)]
    pub async fn create(channels: WorkerMsgChannels) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn start_service(self) -> JoinHandle<()> {
        tokio::spawn(self.service())
    }

    #[allow(unused_mut)]
    async fn service(mut self) {}
}
