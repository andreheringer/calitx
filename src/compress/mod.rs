mod timeseries;
mod gorilla;

extern crate bitvec;
extern crate chrono;
extern crate serde_json;

use bitvec::prelude::*;
use chrono::{Duration, NaiveDateTime};

use std::error::Error;
use std::io::BufRead;

use timeseries::{DateDataPoint, TimeSerie};

pub fn from_reader<R: BufRead>(reader: R, time_batch_interval: i64) -> Result<(), Box<dyn Error>> {
    let interval = Duration::seconds(time_batch_interval);
    let raw: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
    let c = gorilla_compress(&raw, interval);
    Ok(())
}

fn gorilla_compress<'dp>(
    raw_data_points: &'dp Vec<DateDataPoint>,
    time_batch_interval: Duration,
) -> Result<(), Box<dyn Error>> {
    let ts = TimeSerie::new(raw_data_points, time_batch_interval)?;
    for batch in &ts.data_points {
        if let Some(b) = batch {
            let mut payload = BitVec::<Msb0, u32>::from_element(b[0].date_time.timestamp() as u32);
            println!("{:?}", payload);
            let p = gorilla_compress_batch(b, ts.start_date);
            print!("{:?}", p);
        }
    }
    Ok(())
}

fn gorilla_compress_batch(points: &[DateDataPoint], header: NaiveDateTime) -> BitVec<Msb0, u32> {
    let mut payload = BitVec::from_element(header.timestamp() as u32);
    let mut dref = points[0].date_time.signed_duration_since(header);
    let mut href = points[0].date_time;
    payload.append(&mut BitVec::<Msb0, u32>::from_element(
        dref.clone().num_seconds() as u32,
    ));

    for i in 1..points.len() {
        let delta: Duration = points[i].date_time.signed_duration_since(href);
        let dod: i64 = (delta - dref).num_seconds();
        payload.append(&mut gorilla::compress_ts(dod));
        href = points[i].date_time;
        dref = delta;
    }
    payload
}
