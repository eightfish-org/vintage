use anyhow::anyhow;
use serde::Serialize;
use vintage_msg::Act;
use vintage_utils::{BincodeDeserialize, BincodeSerialize};

pub(crate) trait MsgKind {
    fn msg_kind() -> u8;
}

macro_rules! impl_msg_kind {
    (
        $($msg_type:ty = $msg_kind:literal;)*
    ) => {
        $(
            impl MsgKind for $msg_type {
                fn msg_kind() -> u8 {
                    $msg_kind
                }
            }

        )*
    };
}

impl_msg_kind! {
    Act = 1;
}

pub(crate) fn msg_encode<TMsg>(msg: &TMsg) -> anyhow::Result<Vec<u8>>
where
    TMsg: MsgKind + Serialize,
{
    let mut bytes = vec![TMsg::msg_kind()];
    bytes.extend(msg.bincode_serialize()?);
    Ok(bytes)
}

pub(crate) enum MsgDecoded {
    Act(Act),
}

pub(crate) fn msg_decode(msg_encoded: &[u8]) -> anyhow::Result<MsgDecoded> {
    let msg_kind = msg_encoded[0];
    let msg_body = &msg_encoded[1..];
    if msg_kind == Act::msg_kind() {
        let (act, _bytes_read) = Act::bincode_deserialize(msg_body)?;
        Ok(MsgDecoded::Act(act))
    } else {
        Err(anyhow!("invalid msg kind: {}", msg_kind))
    }
}
