mod config;
mod tsparse;

use std::process;
use std::env;
use std::error::Error;

use tsparse::TimeSerie;

fn main() -> Result<(), Box<dyn Error>> {
    
    let args: Vec<String> = env::args().collect();
    
    let conf = config::Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let chunks = TimeSerie::parse_time_series(conf.entry_file_path).unwrap_or_else(|_err| {
        panic!("Oh nwooo.");
    });

    for chunk in chunks.into_iter() {
        println!("{:?}", chunk);
    }

    Ok(())
}
