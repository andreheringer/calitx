pub struct Config {
    pub num_threads: u32,
    pub entry_file_path: String
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enought arguments")
        }

        let num_threads = args[1].parse::<u32>().unwrap();
        let entry_file_path = args[2].clone();
        
        Ok(Config { num_threads, entry_file_path })
    }
}