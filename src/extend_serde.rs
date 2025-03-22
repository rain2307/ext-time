use serde::{de, Deserialize, Deserializer, Serializer};
use time::OffsetDateTime;

/// serde serialize OffsetDateTime to Timestamp
///
/// `#[serde(serialize_with = "serde_t2ts"]`
pub fn serde_t2ts<S>(x: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(x.unix_timestamp())
}

/// serde deserialize Timestamp to OffsetDateTime
///
/// `#serde[(deserialize_with = "serde_parse_ts")]`
pub fn serde_parse_ts<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let t: i64 = Deserialize::deserialize(deserializer)?;
    OffsetDateTime::from_unix_timestamp(t).map_err(de::Error::custom)
}
