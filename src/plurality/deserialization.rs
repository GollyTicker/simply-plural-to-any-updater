
use serde::{Deserialize, Deserializer};

pub fn deserialize_non_empty_string_as_option<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.filter(|s| !s.trim().is_empty()))
}

pub fn parse_epoch_millis_to_datetime_utc<'de, D>(
    d: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let epoch_millis = i64::deserialize(d)?;
    chrono::DateTime::from_timestamp_millis(epoch_millis)
        .ok_or_else(|| serde::de::Error::custom("Datime<Utc> from timestamp failed"))
}
