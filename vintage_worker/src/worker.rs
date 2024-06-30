use vintage_msg::WorkerMsgChannels;

pub struct Worker {}

impl Worker {
    #[allow(unused_variables)]
    pub fn create(channels: WorkerMsgChannels) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
