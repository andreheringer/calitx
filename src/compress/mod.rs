mod gorilla;
mod rhesus;

extern crate chrono;
extern crate serde_json;

use std::error::Error;

pub trait Gorilla {
    fn compress(&self, output_file_path: String, interval: i64) -> Result<usize, Box<dyn Error>>;
}

pub trait Rhesus {
    fn compress(&self, output_file_path: String, interval: i64) -> Result<usize, Box<dyn Error>>;
}

//Change implementation here to return the compressed vector
// for streams.
/* pub fn rhesus_from_reader<R: BufRead>(
    reader: R,
    interval: Duration,
    output_file_path: String,
) -> Result<usize, Box<dyn Error>> {

} */

//Change implementation here to return the compressed vector
// for streams.
/* pub fn gorilla_from_reader<R: BufRead>(
    reader:
) -> Result<usize, Box<dyn Error> {

} */
