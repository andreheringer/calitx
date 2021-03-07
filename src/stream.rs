extern crate chrono;
extern crate fasthash;
extern crate serde;
extern crate serde_json;

use crate::errors::RstzError;

use chrono::{DateTime, Datelike, Timelike, Utc};
use fasthash::city::hash64;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::BTreeMap;
use serde_json::ser::to_string;

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

  pub fn serieskey(&self) -> [u8; 16] {
    let mut res: [u8; 16] = [0; 16];
    res[..8].copy_from_slice(&hash64(self.host.as_bytes()).to_ne_bytes());
    let year: [u8; 4] = self.timestamp.naive_local().date().year().to_ne_bytes();
    res[8..12].copy_from_slice(&year);
    let md = [
      self.timestamp.naive_local().date().month() as u8,
      self.timestamp.naive_local().date().day() as u8,
    ];
    res[12..14].copy_from_slice(&md);
    let hms = [
      self.timestamp.naive_local().time().hour() as u8,
      self.timestamp.naive_local().time().minute() as u8,
    ];
    res[14..].copy_from_slice(&hms);
    res
  }

  pub fn to_string(&self) -> String {
    to_string(self).unwrap_or_default()
  }


}

impl fmt::Display for LogEvent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({}:{} {:?})", self.host, self.timestamp, self.values)
  }
}


pub fn stream_from_file<'fs>(
  file: &'fs File,
) -> serde_json::StreamDeserializer<'fs, serde_json::de::IoRead<BufReader<&File>>, LogEvent>
{
  let reader = BufReader::new(file);
  let deserializer = serde_json::Deserializer::from_reader(reader);
  deserializer.into_iter::<LogEvent>()
}
