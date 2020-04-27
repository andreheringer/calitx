extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::Duration;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::Value;

use std::error::Error;

mod date_serializer {

    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    fn time_to_json(t: NaiveDateTime) -> String {
        DateTime::<Utc>::from_utc(t, Utc).to_rfc3339()
    }

    pub fn serialize<S: Serializer>(
        time: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        time_to_json(time.clone()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(NaiveDateTime::parse_from_str(&time, FORMAT).map_err(D::Error::custom)?)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DateDataPoint {
    #[serde(with = "date_serializer")]
    pub date_time: NaiveDateTime,
    pub value: Value,
}

#[derive(Debug)]
pub struct TimeSerie<'dp> {
    pub time_batch_interval: Duration,
    pub data_points: Vec<&'dp [DateDataPoint]>,
}

impl<'dp> TimeSerie<'dp> {
    pub fn new(
        raw_data_points: &'dp Vec<DateDataPoint>,
        time_batch_interval: Duration,
    ) -> Result<TimeSerie<'dp>, Box<dyn Error>> {
        let mut data_points: Vec<&[DateDataPoint]> = Vec::new();
        let mut start = 0;
        for i in 0..raw_data_points.len() {
            if raw_data_points[i]
                .date_time
                .signed_duration_since(raw_data_points[start].date_time)
                >= time_batch_interval
            {
                data_points.push(&raw_data_points[start..i]);
                start = i;
            }
        }
        if raw_data_points.len() > 0 {
            data_points.push(&raw_data_points[start..]);
        }
        Ok(TimeSerie {
            time_batch_interval,
            data_points,
        })
    }
}
