mod config;
mod errors;
mod tsparse;

extern crate serde_json;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use config::Config;
use tsparse::{Datapoint, TimeSerie};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let cfg = Config::new(&args)?;
    let input_file = File::open(cfg.input_file_path)?;
    let reader = BufReader::new(input_file);
    let raw_data_points: Vec<Datapoint> = serde_json::from_reader(reader)?;
    let time_serie = TimeSerie::new(&raw_data_points, cfg.time_batch_size)?;
    for x in time_serie.data_points.into_iter() {
        println!("{:?}", x);
    }
    Ok(())
}
