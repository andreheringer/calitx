use super::value_decoder::ValueDecoder;
use bitvec::prelude::*;
use std::convert::TryFrom;
use chrono::{Duration, DateTime, Utc, TimeZone};
use crate::events::DataPoint;

pub struct TSDecoder<'s, D: ValueDecoder> {
    block: BitVec<Msb0, u8>,
    value_decoder: D,
    curtime: DateTime<Utc>,
    bitptr: &'s BitSlice<Msb0, u8>,
}

impl<'s, D> TSDecoder<'s, D>
where
    D: ValueDecoder,
{
    pub fn new(src: &[u8]) -> Self {
        let block: BitVec<Msb0, u8> =
            BitVec::from_slice(src).expect("Slice to BitVec convertion error");
        let curtime = Utc.timestamp(
            i64::from_be_bytes(*<&[u8; 8]>::try_from(&block.as_raw_slice()[..8]).unwrap()),
            0,
        );
        let bitptr = &block[64..];
        TSDecoder {
            block,
            value_decoder: ValueDecoder::new(),
            curtime,
            bitptr,
        }
    }
}

impl<D> Iterator for TSDecoder<D>
where
    D: ValueDecoder,
{
    type Item = DataPoint;

    fn next(&mut self) -> Option<DataPoint> {
        
    }
}