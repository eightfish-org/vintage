use vintage_msg::ConsensusMsgChannels;

pub struct Consensus {}

impl Consensus {
    #[allow(unused_variables)]
    pub fn create(channels: ConsensusMsgChannels) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
