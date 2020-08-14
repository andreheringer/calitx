mod rhesus;

extern crate chrono;
extern crate serde_json;

use crate::timeseries::DateDataPoint;
use chrono::Duration;
use std::error::Error;
use std::io::BufRead;

pub fn rhesus_from_reader<R: BufRead>(
    reader: R,
    interval: Duration,
    out_file_name: String,
) -> Result<usize, Box<dyn Error>> {
    let raw: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
    info!("Starting Compression...");
    rhesus::compress(&raw, interval, out_file_name)
}


