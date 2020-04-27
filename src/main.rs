mod config;
mod errors;
mod compress;

extern crate serde_json;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let cfg = Config::new(&args)?;
    let input_file = File::open(cfg.input_file_path)?;
    let reader = BufReader::new(input_file);
    let r = compress::from_reader(reader, cfg.time_batch_size)?;
    Ok(())
}
