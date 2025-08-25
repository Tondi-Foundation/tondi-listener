use std::ops::Deref;

use diesel::{
    deserialize::{FromSql, FromSqlRow, Result as DResult},
    pg::{Pg, PgValue},
    sql_types::Binary,
};
use hex::FromHex;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize, FromSqlRow)]
#[serde(transparent, rename_all = "camelCase")]
#[repr(transparent)]
pub struct Hex {
    pub inner: String,
}

impl Hex {
    pub fn decode(&self) -> Result<Vec<u8>> {
        Vec::<u8>::from_hex(&self.inner)
            .map_err(|e| crate::error::Error::InternalServerError(format!("Invalid hex: {}", e)))
    }
}

impl Deref for Hex {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<String> for Hex {
    fn from(inner: String) -> Self {
        Self { inner }
    }
}

impl From<Hex> for String {
    fn from(hex: Hex) -> Self {
        hex.inner
    }
}

impl FromSql<Binary, Pg> for Hex {
    fn from_sql(value: PgValue) -> DResult<Self> {
        let bytes = value.as_bytes();
        let hex_string = hex::encode(bytes);
        Ok(hex_string.into())
    }
}
