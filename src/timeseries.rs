extern crate chrono;
extern crate serde;
extern crate serde_json;

use crate::errors::CompressionError;

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
//#[serde(rename_all = "camelCase")]
pub struct DateDataPoint {
    #[serde(with = "date_serializer")]
    pub date_time: NaiveDateTime,
    #[serde(with = "date_serializer")]
    pub tpep_dropoff_datetime: NaiveDateTime,
    pub passenger_count: i64,
}

#[derive(Debug)]
pub struct TimeSerie<'dp> {
    pub batch_interval: Duration,
    pub data_points: Vec<Option<&'dp [DateDataPoint]>>,
    pub start_date: NaiveDateTime,
}

impl<'dp> TimeSerie<'dp> {
    pub fn new(
        raw: &'dp Vec<DateDataPoint>,
        batch_interval: Duration,
    ) -> Result<TimeSerie<'dp>, Box<dyn Error>> {
        let start_date = raw[0].date_time.clone().date().and_hms(0, 0, 0);
        let mut data_points: Vec<Option<&[DateDataPoint]>> = Vec::new();
        let mut footer = start_date.checked_add_signed(batch_interval).unwrap();
        let mut start = 0;

        for i in 0..raw.len() {
            while raw[start].date_time.signed_duration_since(footer) >= Duration::zero() {
                data_points.push(None);
                // TODO: Remove this unwrap
                footer = footer.checked_add_signed(batch_interval).unwrap();
            }

            if raw[i].date_time.signed_duration_since(footer) >= Duration::zero() {
                data_points.push(Some(&raw[start..i - 1]));
                start = i;
                // TODO: Remove this unwrap
                footer = footer.checked_add_signed(batch_interval).unwrap();
            }
        }
        if raw.len() > 0 {
            data_points.push(Some(&raw[start..]));
        }

        Ok(TimeSerie {
            batch_interval,
            data_points,
            start_date,
        })
    }
}
