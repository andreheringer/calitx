extern crate chrono;
extern crate getopts;

use getopts::Matches;
use getopts::Options;

use crate::errors::ConfigError;
use std::error::Error;
use std::str::FromStr;

pub struct Config {
    pub num_threads: u32,
    pub input_file_path: String,
    pub output_file_path: String,
    pub time_batch_size: i64,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        let mut opts: Options = Options::new();
        opts.optopt(
            "o",
            "",
            "The path of the output file including the file extension.",
            "TARGET_FILE_NAME",
        );
        opts.optopt(
            "n",
            "",
            "Number of threads used for the file compression.",
            "NUM_THREADS",
        );
        opts.optflag("h", "help", "Prints this help menu.");
        opts.optopt(
            "",
            "batch",
            "batch time window for compression",
            "BATCH_SIZE",
        );

        let matches: Matches = opts.parse(&args[1..])?;

        let input_file_path: String = if matches.free.is_empty() {
            return Err(Box::new(ConfigError::new("No input file arg.")));
        } else {
            matches.free[0].clone()
        };

        let output_file_path = matches.opt_str("o").unwrap_or(String::from_str("out")?);
        let num_threads = matches
            .opt_str("n")
            .unwrap_or(String::from_str("1")?)
            .parse::<u32>()?;

        let time_batch_size = 
            matches
                .opt_str("batch")
                .unwrap_or(String::from_str("7200")?)
                .parse::<i64>()?;

        if !(input_file_path.contains(".json")) {
            return Err(Box::new(ConfigError::new(
                "Input file must be json.",
            )));
        }

        Ok(Config {
            num_threads,
            input_file_path,
            output_file_path,
            time_batch_size,
        })
    }
}
