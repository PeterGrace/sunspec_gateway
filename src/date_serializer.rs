use chrono::{DateTime, SecondsFormat, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

pub fn time_to_json(time: DateTime<Utc>) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(time.naive_utc(), Utc)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn serialize<S: Serializer>(time: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
    time_to_json(time.clone()).serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Utc>, D::Error> {
    let time: String = Deserialize::deserialize(deserializer)?;
    Ok(DateTime::<Utc>::from_str(&time).map_err(D::Error::custom)?)
}
