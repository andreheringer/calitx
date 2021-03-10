mod compress;
mod errors;
mod stream;
mod tree;

use compress::xdelta;
use std::fs::File;

fn main() {
  let inpt = File::open("test.json").unwrap();
  let file_stream = stream::stream_from_file(&inpt);
  for event in file_stream {
    println!("{}", event.unwrap().to_string());
  }
  
}
