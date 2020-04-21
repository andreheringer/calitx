extern crate chrono;
extern crate getopts;

use chrono::Duration;
use getopts::Matches;
use getopts::Options;

use crate::errors::ConfigError;
use std::error::Error;
use std::str::FromStr;

pub struct Config {
    pub num_threads: u32,
    pub input_file_path: String,
    pub output_file_path: String,
    pub is_nulled: bool,
    pub is_keyed: bool,
    pub time_batch_size: Duration,
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
        opts.optflag("", "null", "Empty strings are set to null.");
        opts.optflag("", "no-keyed", "Generate output as keyed JSON.");
        opts.optflag("h", "help", "Prints this help menu.");
        opts.optopt("", "chunks", "Chunk size for compression", "CHUNK_SIZE");

        let matches: Matches = opts.parse(&args[1..])?;

        let input_file_path: String = if matches.free.is_empty() {
            return Err(Box::new(ConfigError::new("No input file arg.")));
        } else {
            matches.free[0].clone()
        };

        let output_file_path = matches
            .opt_str("o")
            .unwrap_or(String::from_str("out.json")?);
        let num_threads = matches
            .opt_str("n")
            .unwrap_or(String::from_str("out.json")?)
            .parse::<u32>()?;
        let is_keyed = !matches.opt_present("no-keyed");
        let is_nulled = matches.opt_present("null");

        let time_batch_size = Duration::seconds(
            matches
                .opt_str("chunks")
                .unwrap_or(String::from_str("7200")?)
                .parse::<i64>()?
        );

        if !(input_file_path.contains(".json") || input_file_path.contains(".csv")) {
            return Err(Box::new(ConfigError::new(
                "Input file must be either csv or json.",
            )));
        }

        Ok(Config {
            num_threads,
            input_file_path,
            output_file_path,
            is_nulled,
            is_keyed,
            time_batch_size,
        })
    }
}
