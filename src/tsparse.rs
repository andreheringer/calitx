extern crate serde_json;

use serde_json::Value as JsonValue;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug)]
pub struct TimeSerie {
    datetime: String,
    entry: String
}

impl TimeSerie {
    
    fn new(datetime: String, entry: String) -> TimeSerie {
        TimeSerie {
            datetime: datetime,
            entry: entry
        }
    }

    pub fn parse_time_series<P: AsRef<Path>>(path: P) -> Result<Vec<TimeSerie>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
    
        let desirializer = serde_json::Deserializer::from_reader(reader);
        let mut chunks: Vec<TimeSerie> = Vec::new();
    
        for value in desirializer.into_iter::<JsonValue>() {
            let res = value.unwrap_or_else(|_err| {
                panic!("Something went wrong.");
            });
            chunks.push(TimeSerie::new(res["datetime"].to_string(), res["entry"].to_string()));
        }
    
        Ok(chunks)
        
    }
}

// TODO: Create a HashMap that segments the input file in chunks of 2 hours
