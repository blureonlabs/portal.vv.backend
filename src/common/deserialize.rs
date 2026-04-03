use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

/// Deserialize empty strings as None for Option<NaiveDate>
pub fn empty_string_as_none_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<NaiveDate>, D::Error> {
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => s.parse::<NaiveDate>().map(Some).map_err(serde::de::Error::custom),
    }
}

/// Deserialize empty strings as None for Option<Uuid>
pub fn empty_string_as_none_uuid<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Uuid>, D::Error> {
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => s.parse::<Uuid>().map(Some).map_err(serde::de::Error::custom),
    }
}

/// Deserialize empty strings as None for Option<String>
pub fn empty_string_as_none_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => Ok(Some(s.to_owned())),
    }
}
