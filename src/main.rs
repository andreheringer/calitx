use std::process;
use std::env;
use std::error::Error;
mod config;
mod tsparse;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let conf = config::Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    tsparse::parse_time_series(conf.entry_file_path)
}
