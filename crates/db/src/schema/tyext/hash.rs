use hex::FromHex;

use crate::error::Error;

pub type Hash<const N: usize> = [u8; N];

pub type Hash256 = Hash<32>;

pub trait FromHexString<T>: Sized {
    type Error;

    fn from_hex(hex: T) -> Result<Self, Self::Error>;
}

impl<T, const N: usize> FromHexString<T> for Hash<N>
where
    T: AsRef<[u8]>,
{
    type Error = Error;

    fn from_hex(hex: T) -> Result<Self, Self::Error> {
        let hex_str = std::str::from_utf8(hex.as_ref())
            .map_err(|e| Error::InternalServerError(format!("Invalid UTF-8: {}", e)))?;
        
        let bytes = Vec::<u8>::from_hex(hex_str)
            .map_err(|e| Error::InternalServerError(format!("Invalid hex: {}", e)))?;
        
        if bytes.len() != N {
            return Err(Error::InternalServerError(format!("Expected {} bytes, got {}", N, bytes.len())));
        }
        
        let mut dst = [0; N];
        dst.copy_from_slice(&bytes);
        Ok(dst)
    }
}
