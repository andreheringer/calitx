extern crate chrono;
extern crate fasthash;
extern crate serde;
extern crate serde_json;

use chrono::{Date, DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;
use std::fmt;
use std::{fs::File, io::BufReader};

///Most basic implementation of a Log Event, contains the same caracteristics defined by vector.
///Timestamp and host fields are requierd.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEvent {
    timestamp: DateTime<Utc>,
    host: String,

    #[serde(flatten)]
    values: BTreeMap<String, Value>,
}

impl LogEvent {
    pub fn new(timestamp: DateTime<Utc>, host: String, values: BTreeMap<String, Value>) -> Self {
        LogEvent {
            timestamp,
            host,
            values,
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::ser::to_string(self).unwrap_or_default()
    }

    pub fn date(&self) -> Date<Utc> {
        self.timestamp.date()
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl fmt::Display for LogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{} {:?})", self.host, self.timestamp, self.values)
    }
}

pub fn stream_from_file<'fs>(
  file: &'fs File,
) -> serde_json::StreamDeserializer<'fs, serde_json::de::IoRead<BufReader<&File>>, LogEvent> {
    let reader = BufReader::new(file);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    deserializer.into_iter::<LogEvent>()
}

/// Basic DataPoint representation
pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: Value,
}

impl DataPoint {
    pub fn new(timestamp: DateTime<Utc>, value: Value) -> Self {
        DataPoint { timestamp, value }
    }
}
