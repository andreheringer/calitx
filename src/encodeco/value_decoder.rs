use serde_json::Value;
use bitvec::prelude::*;

pub trait ValueDecoder {
    fn new() -> Self;
    fn decompress(&mut self, bitptr: &mut BitSlice<Msb0, u8>) -> Value;
}
