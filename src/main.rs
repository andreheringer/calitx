mod compress;
mod config;
mod errors;

extern crate serde_json;
#[macro_use]
extern crate log;
extern crate simplelog;

use config::Config as RstzConfig;
use simplelog::*;
use std::env;
use std::error::Error;
use std::fs::File;


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


    Ok(())
}
