use std::marker::PhantomData;

use bytes::{Buf, BufMut, buf::Writer};
use nill::{Nil, nil};
use rkyv::{
    Archive, Deserialize, Serialize,
    api::high::{HighDeserializer, HighSerializer, to_bytes_in},
    from_bytes_unchecked,
    rancor::Error as CodecError,
    ser::{allocator::ArenaHandle, writer::IoWriter},
};
use tonic::{
    Status,
    codec::{Codec as ICodec, DecodeBuf, Decoder as IDecoder, EncodeBuf, Encoder as IEncoder},
};

#[derive(Debug)]
pub struct Codec<T, U> {
    maker: PhantomData<(T, U)>,
}

impl<T, U> Default for Codec<T, U> {
    fn default() -> Self {
        Self { maker: PhantomData }
    }
}

pub type EWriter<'a, 'b> = IoWriter<Writer<&'a mut EncodeBuf<'b>>>;

pub type ESerializer<'a, 'b, 'c> = HighSerializer<EWriter<'a, 'b>, ArenaHandle<'c>, CodecError>;

pub type DSerializer = HighDeserializer<CodecError>;

impl<T, U> ICodec for Codec<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
    T: for<'a, 'b, 'c> Serialize<ESerializer<'a, 'b, 'c>>,
    U: Archive,
    U::Archived: Deserialize<U, DSerializer>,
{
    type Decode = U;
    type Decoder = Decoder<U>;
    type Encode = T;
    type Encoder = Encoder<T>;

    fn encoder(&mut self) -> Self::Encoder {
        Self::Encoder::new()
    }

    fn decoder(&mut self) -> Self::Decoder {
        Self::Decoder::new()
    }
}

#[derive(Debug)]
pub struct Encoder<T> {
    marker: PhantomData<T>,
}

impl<T> Encoder<T> {
    pub const fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<T> IEncoder for Encoder<T>
where
    T: for<'a, 'b, 'c> Serialize<ESerializer<'a, 'b, 'c>>,
{
    type Error = Status;
    type Item = T;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<Nil, Self::Error> {
        let writer = buf.writer();
        to_bytes_in(&item, IoWriter::new(writer)).map_err(Error::Codec)?;
        Ok(nil)
    }
}

#[derive(Debug, Default)]
pub struct Decoder<U> {
    marker: PhantomData<U>,
}

impl<U> Decoder<U> {
    pub const fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<U> IDecoder for Decoder<U>
where
    U: Archive,
    U::Archived: Deserialize<U, DSerializer>,
{
    type Error = Status;
    type Item = U;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let bytes = buf.chunk();
        let value = unsafe { from_bytes_unchecked::<U, CodecError>(bytes) };
        buf.advance(bytes.len());
        Ok(Some(value.map_err(Error::Codec)?))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Codec(#[from] CodecError),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        Status::internal(format!("{err}"))
    }
}
