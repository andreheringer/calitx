extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::Duration;
use chrono::NaiveDateTime;
use serde::Deserialize;
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
pub struct DateDataPoint {
    #[serde(with = "date_serializer")]
    pub time: NaiveDateTime,
    pub value: f32,
}

#[derive(Debug)]
pub struct TimeBatch<'dp> {
    pub header: NaiveDateTime,
    pub points: &'dp [DateDataPoint],
}

#[derive(Debug)]
pub struct TimeSerie<'dp> {
    pub batch_interval: Duration,
    pub batches: Vec<TimeBatch<'dp>>,
}

impl<'dp> TimeSerie<'dp> {
    pub fn new(
        raw: &'dp Vec<DateDataPoint>,
        batch_interval: Duration,
    ) -> Result<TimeSerie<'dp>, Box<dyn Error>> {
        debug!("#### Building Time Series Chuncks...");
        let mut batches: Vec<TimeBatch> = Vec::new();
        let mut header = raw[0].time.clone().date().and_hms(0, 0, 0);
        let mut p = 0;
        while p < raw.len() {
            while raw[p].time.signed_duration_since(header) > batch_interval {
                header = header.checked_add_signed(batch_interval).unwrap();
            }
            debug!("Found a Header: {:?}", header);
            let start = p;
            let mut end = start;
            while end < raw.len() && raw[end].time.signed_duration_since(header) <= batch_interval {
                end += 1;
            }
            debug!("Batch entries ranging from {:?} to {:?}", start, end - 1);
            batches.push(TimeBatch {
                header: header,
                points: &raw[start..end],
            });
            p = end;
        }

        Ok(TimeSerie {
            batch_interval,
            batches,
        })
    }
}