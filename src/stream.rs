extern crate serde_json;

use crate::errors::RstzError;
use crate::log_event::LogEvent;

use std::{fs::File, io::BufReader, path::Path};


pub fn read_log_events_from_file<P: AsRef<Path>>(path: P) -> Result<LogEvent, RstzError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader).map_err(|e| RstzError::Message(e.to_string()))?;

    // Return the `User`.
    Ok(u)
}
