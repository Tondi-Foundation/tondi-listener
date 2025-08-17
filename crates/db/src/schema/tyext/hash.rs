use hex::hex_decode;

use crate::error::Error;

pub type Hash<const N: usize> = [u8; N];

pub type Hash256 = Hash<32>;

pub trait FromHex<T>: Sized {
    type Error;

    fn from_hex(hex: T) -> Result<Self, Self::Error>;
}

impl<T, const N: usize> FromHex<T> for Hash<N>
where
    T: AsRef<[u8]>,
{
    type Error = Error;

    fn from_hex(hex: T) -> Result<Self, Self::Error> {
        let mut dst = [0; N];
        hex_decode(hex.as_ref(), &mut dst)?;
        Ok(dst)
    }
}
