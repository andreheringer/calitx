mod compress;
mod config;
mod errors;
mod timeseries;
mod wip;

extern crate serde_json;
#[macro_use]
extern crate log;
extern crate simplelog;

use chrono::NaiveDateTime;
use config::Config as RstzConfig;
use simplelog::*;
use std::env;
use std::error::Error;
use std::{fs::File};
use wip::TimedDataPoint;

mod chrono_serializer {

    use chrono::NaiveDateTime;
    use serde::{de::Error, Deserialize, Deserializer};
    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(NaiveDateTime::parse_from_str(&time, FORMAT).map_err(D::Error::custom)?)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DateTuple {
    #[serde(with = "chrono_serializer")]
    pub timestamp: chrono::NaiveDateTime,
    pub value: serde_json::Value,
}

impl TimedDataPoint for DateTuple {
    fn naivetime(&self) -> NaiveDateTime {
        return self.timestamp;
    }

    fn value(&self) -> serde_json::Value {
        return self.value.clone();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("rst_run.log").unwrap(),
        ),
    ])
    .unwrap();
    info!("Initiating RSTZ...");
    let args: Vec<String> = env::args().collect();
    let cfg = RstzConfig::new(&args)?;
    info!("Starting RSTZ with parameters:\nInput file: {:?}\nBatch Time Window (in secs): {:?}\nNumber of Threads: {:?}\nOutput File {:?}", 
        cfg.input_file_path,
        cfg.time_batch_size,
        cfg.num_threads,
        cfg.output_file_path
    );

    let input_file = File::open(cfg.input_file_path)?;
    let mut output_file = File::create(&cfg.output_file_path)?;
    let res = wip::gorilla::compress_from_file::<DateTuple>(
        &input_file,
        &mut output_file,
        cfg.time_batch_size,
    );
    info!("{:?}", res?);
    Ok(())
}
