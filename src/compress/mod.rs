mod rhesus;
mod gorilla;

extern crate chrono;
extern crate serde_json;

use crate::timeseries::DateDataPoint;
use chrono::Duration;
use std::error::Error;
use std::io::BufRead;

pub trait Gorilla {
    fn compress(&self, output_file_path: String, interval: i64) -> Result<usize, Box<dyn Error>>;
}

//Change implementation here to return the compressed vector
// for streams.
pub fn rhesus_from_reader<R: BufRead>(
    reader: R,
    interval: Duration,
    output_file_path: String,
) -> Result<usize, Box<dyn Error>> {
    let raw: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
    info!("Starting Compression...");
    rhesus::compress(&raw, interval, output_file_path)
}

//Change implementation here to return the compressed vector
// for streams.
/* pub fn gorilla_from_reader<R: BufRead>(
    reader:
) -> Result<usize, Box<dyn Error> {

} */
