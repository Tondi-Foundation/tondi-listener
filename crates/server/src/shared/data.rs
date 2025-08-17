use std::{error::Error as StdError, mem};

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::error::Error;

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Fail,
}

impl Status {
    pub const MAX: u8 = (mem::variant_count::<Self>() - 1) as u8;
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Safe: std::mem::discriminant(_)
        let code = unsafe { *<*const Self>::from(self).cast::<u8>() };
        Serialize::serialize(&code, serializer)
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u8::deserialize(deserializer)? {
            code @ 0..=Self::MAX => {
                // Safe: Status in (0..=MAX)
                let status = unsafe { mem::transmute(code) };
                Ok(status)
            },
            rest => Err(de::Error::custom(format!("Invalid Status: {rest} > {}", Self::MAX))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner<T> {
    pub status: Status,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
}

impl<T> Inner<T> {
    pub fn new(data: T) -> Self {
        Self { status: Status::Ok, data: Some(data), cause: None }
    }

    pub fn fail(cause: String) -> Self {
        Self { status: Status::Fail, data: None, cause: Some(cause) }
    }
}

impl<T> From<T> for Inner<T>
where
    T: Serialize,
{
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T, E> From<Result<T, E>> for Inner<T>
where
    E: StdError,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => Self::new(data),
            Err(err) => Self::fail(format!("{err}")),
        }
    }
}

impl<T> IntoResponse for Inner<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl<T> From<Inner<T>> for Response
where
    T: Serialize,
{
    fn from(inner: Inner<T>) -> Self {
        inner.into_response()
    }
}

pub type Data<T, E = Error> = Result<Inner<T>, E>;
