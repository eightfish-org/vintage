use vintage_msg::NetworkMsgChannels;

pub struct Network {}

impl Network {
    #[allow(unused_variables)]
    pub fn create(channels: NetworkMsgChannels) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
