extern crate serde_json;

use serde_json::Value as JsonValue;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;


//pub struct TimeSerie {
//    datetime: String,
//    entry: i32
//}

// impl TimeSerie {
//     pub fn parse_time_series<P: AsRef<Path>>(path: P) {
//         let file = File::open(path).unwrap();
//         let reader = BufReader::new(file);
    
//         for (index, line) in reader.lines().enumerate() {
//             let line = line.unwrap();
//             println!("{} {}", index + 1, line);
//         }
//     }
// }

pub fn parse_time_series<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let desirializer = serde_json::Deserializer::from_reader(reader);

    for value in desirializer.into_iter::<JsonValue>() {
        let res = value.unwrap_or_else(|_err| {
            panic!("Something went wrong.");
        });
        println!("Date: {}", res["datetime"]);
        println!("Entry: {}", res["entry"]);
    }

    Ok(())
    
}