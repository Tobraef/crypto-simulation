use serde::Serialize;

use anyhow::Result;

pub fn serialize<T>(serializable: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    serde_json::to_vec(serializable).map_err(|e| e.into())
}
