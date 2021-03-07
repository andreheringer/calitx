mod compressor;
mod delta;

use compressor::DeltaCompressor;
use delta::{Delta};


pub fn xdelta(source: &str, target: &str) -> Delta {
    let mut compressor = DeltaCompressor::new(source.as_bytes(), target.as_bytes());
    compressor.compress();
    compressor.gendelta()
}
