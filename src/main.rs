mod compress;
mod config;
mod errors;
mod timeseries;

extern crate serde_json;

use config::Config;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let cfg = Config::new(&args)?;
    let input_file = File::open(cfg.input_file_path)?;
    let reader = BufReader::new(input_file);
    compress::from_reader(reader, cfg.time_batch_size, cfg.output_file_path)?;
    Ok(())
}
