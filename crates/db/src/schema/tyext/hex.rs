use std::ops::Deref;

use diesel::{
    deserialize::{FromSql, FromSqlRow, Result as DResult},
    pg::{Pg, PgValue},
    sql_types::Binary,
};
use hex::{hex_decode, hex_string};
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
        let mut dst = vec![0; self.len() / 2];
        hex_decode(self.as_bytes(), &mut dst)?;
        Ok(dst)
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
        Ok(hex_string(value.as_bytes()).into())
    }
}
