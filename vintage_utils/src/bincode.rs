use bincode::config::standard;
use bincode::error::{DecodeError, EncodeError};
use serde::de::DeserializeOwned;
use serde::Serialize;

// bincode is a compact encoder / decoder pair that uses a binary zero-fluff encoding scheme.

pub trait BincodeSerialize {
    fn bincode_serialize(&self) -> Result<Vec<u8>, EncodeError>;
    fn bincode_serialize_into_slice(&self, slice: &mut [u8]) -> Result<usize, EncodeError>;
}

pub trait BincodeDeserialize: Sized {
    fn bincode_deserialize(bytes: &[u8]) -> Result<(Self, usize), DecodeError>;
}

impl<T> BincodeSerialize for T
where
    T: Serialize,
{
    fn bincode_serialize(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, standard())
    }
    fn bincode_serialize_into_slice(&self, slice: &mut [u8]) -> Result<usize, EncodeError> {
        bincode::serde::encode_into_slice(self, slice, standard())
    }
}

impl<T> BincodeDeserialize for T
where
    T: DeserializeOwned,
{
    fn bincode_deserialize(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        bincode::serde::decode_from_slice(bytes, standard())
    }
}
