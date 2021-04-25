extern crate bitvec;
extern crate chrono;
extern crate fasthash;
extern crate serde;
extern crate serde_json;

mod encodeco;
mod errors;
mod events;
mod tree;

use chrono::Duration;
use chrono::{TimeZone, Utc};

use std::convert::TryFrom;
use std::fs::File;
use std::string::String;

use encodeco::{GorillaEncoder, TsEncoder};
use errors::Result;

fn main() -> Result<()> {
  let inpt = File::open("test.json").unwrap();
  let file_stream = events::stream_from_file(&inpt);
  let mut encoder =
    TsEncoder::<GorillaEncoder>::new(String::from("value"), Duration::minutes(120));
  
  for it in file_stream {
    if let Ok(event) = it {
      let e = encoder.compress(event)?;
      if let Some(block) = e {
        println!("Block: {:?}", block);
      }
    }
  }

  let r = encoder.genblock();
  println!("Remainder: {:?}", r);

  let f = i64::from_be_bytes(*<&[u8; 8]>::try_from(&r[..8]).unwrap());
  println!("{:?}", f);
  let mut time = Utc.timestamp(f, 0);
  println!("{:?}", time.to_string());
  let d = i64::from_be_bytes(*<&[u8; 8]>::try_from(&r[8..16]).unwrap());
  println!("{:?}", d);
  let d2 = Duration::milliseconds(d);
  time = time + d2;
  println!("{:?}", time.to_string());
  Ok(())
}
