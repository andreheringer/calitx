use super::value_decoder::ValueDecoder;
use bitvec::prelude::*;
use std::convert::TryFrom;
use chrono::{Duration, DateTime, Utc, TimeZone};
use crate::events::DataPoint;

pub struct TSDecoder<'s, D: ValueDecoder> {
    block: BitVec<Msb0, u8>,
    value_decoder: D,
    curtime: Option<DateTime<Utc>>,
    bitptr: Option<&'s BitSlice<Msb0, u8>>,
    last_delta: Option<i64>,
}

impl<'s, D> TSDecoder<'s, D>
where
    D: ValueDecoder,
{
    pub fn new(src: &[u8]) -> Self {
        TSDecoder {
            block: BitVec::from_slice(src).expect("Slice to BitVec convertion error"),
            value_decoder: ValueDecoder::new(),
            curtime: None,
            bitptr: None,
            last_delta: None,
        }
    }

    pub fn decompress(&mut self) -> Option<DataPoint> {
        if let Some(curtime) = self.curtime {

        } else {
            let mut time = Utc.timestamp(
                i64::from_be_bytes(*<&[u8; 8]>::try_from(&self.block.as_raw_slice()[..8]).unwrap()),
                0,
            );
            let mut slice = &self.block.as_bitslice()[64..];
            self.bitptr = Some(slice);
            let delta = i64::from_be_bytes(*<&[u8; 8]>::try_from(&slice.as_raw_slice()[..8]).unwrap());
            time = time + Duration::milliseconds(delta);
            slice = &slice[64..];
            self.bitptr = Some(slice);
            self.curtime = Some(time);
            self.last_delta(delta);
            let value = self.value_decoder.decompress(self.bitptr);
            Some(DataPoint::new(time, value))
        }
    }
}

/* impl<D> Iterator for TSDecoder<D>
where
    D: ValueDecoder,
{
    type Item = DataPoint;

    fn next(&mut self) -> Option<DataPoint> {
        
    }
} */