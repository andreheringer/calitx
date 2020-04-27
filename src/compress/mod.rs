mod timeseries;

extern crate bitvec;
extern crate chrono;
extern crate serde_json;

use bitvec::prelude::*;
use chrono::Duration;

use std::error::Error;
use std::io::BufRead;

use timeseries::{DateDataPoint, TimeSerie};

pub fn from_reader<R: BufRead>(reader: R, time_batch_interval: i64) -> Result<(), Box<dyn Error>> {
    let interval = Duration::seconds(time_batch_interval);
    let raw_data_points: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
    let c = compress(&raw_data_points, interval);
    Ok(())
}

fn compress<'dp>(
    raw_data_points: &'dp Vec<DateDataPoint>,
    time_batch_interval: Duration,
) -> Result<(), Box<dyn Error>> {
    let ts = TimeSerie::new(raw_data_points, time_batch_interval)?;
    for date_time_series in &ts.data_points {
        let header = date_time_series[0].date_time;
        println!("Header {:?}", header);
        for i in 0..date_time_series.len() {
            let y =
                BitVec::<Msb0, u64>::from_element(date_time_series[i].date_time.timestamp() as u64);
            println!("{:?}", y);
        }
        println!("Ended following batch.");
    }

    println!("{:?}", ts);
    Ok(())
}
