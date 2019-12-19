use std::env;
mod logparse;

struct Config {
    num_threads: u32,
    log_filename: String
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3{
            return Err("not enought arguments")
        }

        let num_threads = args[1].parse::<u32>().unwrap();
        let log_filename = args[2].clone();
        
        Ok(Config { num_threads, log_filename })
    }
}

fn main() -> std::io::Result<()> {
    for line in logparse::BufReader::open("Cargo.toml")? {
        println!("{}", line?.trim());
    }

    Ok(())
}

