use crate::messages::NetworkMessage;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub(crate) struct BlockchainCodec;

impl Decoder for BlockchainCodec {
    type Item = NetworkMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            return Ok(None);
        }

        src.advance(4);
        let message_bytes = src.split_to(length);

        match bincode::deserialize(&message_bytes) {
            Ok(message) => Ok(Some(message)),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Encoder<NetworkMessage> for BlockchainCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: NetworkMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let message_bytes = bincode::serialize(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        dst.put_u32(message_bytes.len() as u32);
        dst.extend_from_slice(&message_bytes);

        Ok(())
    }
}
