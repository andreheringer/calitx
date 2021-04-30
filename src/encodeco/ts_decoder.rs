use super::value_decoder::ValueDecoder;
use crate::events::DataPoint;
use bitvec::prelude::*;
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::convert::TryFrom;

enum DtsRange {
    Tinny,
    Small,
    Medium,
    Large
}

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

    pub fn decompress(&'s mut self) -> Option<DataPoint> {
        if let Some(bitslice) = self.bitptr {
            if bitslice.is_empty() {
                return None;
            }
        }
        if let Some(curtime) = self.curtime {
            let mut slice = self.bitptr.unwrap();
            None
        } else {
            let mut time = Utc.timestamp(
                i64::from_be_bytes(*<&[u8; 8]>::try_from(&self.block.as_raw_slice()[..8]).unwrap()),
                0,
            );
            let mut slice = &self.block.as_bitslice()[64..];
            let delta =
                i64::from_be_bytes(*<&[u8; 8]>::try_from(&slice.as_raw_slice()[..8]).unwrap());
            time = time + Duration::milliseconds(delta);
            slice = &slice[64..];
            let value = self.value_decoder.decompress(&mut slice);
            self.curtime = Some(time);
            self.last_delta = Some(delta);
            self.bitptr = Some(slice);
            Some(DataPoint::new(time, value))
        }
    }

    fn decode_dod(slice: &'s mut BitSlice<Msb0, u8>) -> i64 {
        if slice[0] == false {
            slice = &mut slice[1..];
            return 0;
        }
        let range = Self::decode_range(&mut slice);
        match range {
            Tinny => {
                let dod_bits: BitSlice<Msb0, u8> = bits![mut 0; 7];
                dod_bits.copy_from_bitslice(&slice[..7]);
                
                slice = &mut slice[7..];
            }
        }
    }

    fn decode_range(slice: &'s mut BitSlice<Msb0, u8>) -> DtsRange {
        if slice[1] == false {
            slice = &mut slice[2..];
            return DtsRange::Tinny;
        }

        if slice[2] == false {
            slice = &mut slice[3..];
            return DtsRange::Small;
        }

        if slice[3] == false {
            slice = &mut slice[4..];
            return DtsRange::Medium;
        }

        slice = &mut slice[4..];
        DtsRange::Large
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
