extern crate chrono;
extern crate fasthash;
extern crate serde;

use std::fmt;

use chrono::{DateTime, Datelike, Timelike, Utc};
use fasthash::city::hash32;
use serde::Deserialize;
use serde_json::Value;

///Most basic implementation of a Log Event, contains the same caracteristics defined by vector.
///Timestamp and host fields are requierd.
#[derive(Deserialize, Debug, Clone)]
pub struct LogEvent {
    timestamp: DateTime<Utc>,
    host: String,
    value: Value,
}

impl LogEvent {
    pub fn new(timestamp: DateTime<Utc>, host: String, value: Value) -> Self {
        LogEvent {
            timestamp,
            host,
            value,
        }
    }

    pub fn serieskey(&self) -> [u8; 17] {
        let mut res: [u8; 17] = [0; 17];
        res[..4].copy_from_slice(&hash32(self.host.as_bytes()).to_ne_bytes());
        let year: [u8; 4] = self.timestamp.naive_local().date().year().to_ne_bytes();
        res[4..8].copy_from_slice(&year);
        let md = [
            self.timestamp.naive_local().date().month() as u8,
            self.timestamp.naive_local().date().day() as u8,
        ];
        res[8..10].copy_from_slice(&md);
        let hms = [
            self.timestamp.naive_local().time().hour() as u8,
            self.timestamp.naive_local().time().minute() as u8,
            self.timestamp.naive_local().time().second() as u8,
        ];
        res[10..13].copy_from_slice(&hms);
        let nanos = self
            .timestamp
            .naive_local()
            .time()
            .nanosecond()
            .to_ne_bytes();
        res[13..].copy_from_slice(&nanos);
        res
    }
}

impl fmt::Display for LogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{} {})", self.host, self.timestamp, self.value)
    }
}
