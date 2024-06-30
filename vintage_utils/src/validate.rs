use std::fmt::Display;

pub trait Validate {
    type Data;
    type Error: Display;

    fn validate(&self, data: &Self::Data) -> Result<(), Self::Error>;
}
