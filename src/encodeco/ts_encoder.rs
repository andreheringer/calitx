use super::value_encoder::ValueEncoder;
use crate::errors::RstzError;
use crate::events::LogEvent;
use bitvec::prelude::*;
use chrono::{DateTime, Duration, Utc};
use std::ops::Range;

const TINNY_DTS: BitArray<Msb0, [u8; 1]> = bitarr![const Msb0, u8; 1, 0];
const SMALL_DTS: BitArray<Msb0, [u8; 1]> = bitarr![const Msb0, u8; 1, 1, 0];
const MEDIUM_DTS: BitArray<Msb0, [u8; 1]> = bitarr![const Msb0, u8; 1, 1, 1, 0];
const LARGE_DTS: BitArray<Msb0, [u8; 1]> = bitarr![const Msb0, u8; 1, 1, 1, 1];

const TINNY_DTS_RANGE: Range<i64> = -63..65;
const SMALL_DTS_RANGE: Range<i64> = -255..257;
const MEDIUM_DTS_RANGE: Range<i64> = -2047..2049;

const ENCODED_64_7: BitArray<Msb0, [u8; 1]> = bitarr![const Msb0, u8; 1, 0, 0, 0, 0, 0, 0];
const ENCODED_256_9: BitArray<Msb0, [u16; 1]> = bitarr![const Msb0, u16; 1, 0, 0, 0, 0, 0, 0, 0, 0];
const ENCODED_2048_12: BitArray<Msb0, [u16; 1]> =
    bitarr![const Msb0, u16; 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub struct TsEncoder<E: ValueEncoder> {
    interval: Duration,
    field: String,
    value_encoder: E,
    cur_header: Option<DateTime<Utc>>,
    last_delta: Option<i64>,
    last_entry: Option<LogEvent>,
    block: BitVec<Msb0, u8>,
}

impl<E> TsEncoder<E>
where
    E: ValueEncoder,
{
    /// Returns a new encoder that can be used to compress series.
    pub fn new(field: String, interval: Duration) -> Self {
        TsEncoder {
            interval,
            field,
            value_encoder: ValueEncoder::new(),
            cur_header: None,
            last_delta: None,
            last_entry: None,
            block: BitVec::new(),
        }
    }

    pub fn compress(&mut self, entry: LogEvent) -> Result<Option<Vec<u8>>, RstzError> {
        match self.cur_header {
            Some(header) => {
                if entry.datetime().signed_duration_since(header) > self.interval {
                    let result = self.block.clone().into_vec();
                    self.block.clear();
                    self.cur_header = None;
                    Ok(Some(result))
                } else {
                    if let Some(last_delta) = self.last_delta {
                        let last_entry = self
                            .last_entry
                            .as_ref()
                            .expect("Bad gen state encountered.");
                        let delta = entry
                            .datetime()
                            .signed_duration_since(last_entry.datetime())
                            .num_milliseconds();
                        let dod = delta - last_delta;
                        self.encode_dod(dod);
                        let value_encoded =
                            self.value_encoder.compress(self.field.as_str(), &entry)?;
                        if let Some(slice) = value_encoded {
                            self.block.extend_from_bitslice(slice);
                        }
                    }
                    Ok(None)
                }
            }
            None => {
                let mut header = entry.date().and_hms(0, 0, 0);
                while entry.datetime().signed_duration_since(header) > self.interval {
                    header = header
                        .checked_add_signed(self.interval)
                        .ok_or(RstzError::from_none())?;
                }
                let delta = entry
                    .datetime()
                    .signed_duration_since(header)
                    .num_milliseconds();
                self.cur_header = Some(header);
                self.last_delta = Some(delta);
                self.block
                    .extend_from_raw_slice(&header.timestamp().to_be_bytes());
                self.block.extend_from_raw_slice(&delta.to_be_bytes());
                let value_encoded = self.value_encoder.compress(self.field.as_str(), &entry)?;
                if let Some(slice) = value_encoded {
                    self.block.extend_from_bitslice(slice);
                }
                self.last_entry = Some(entry);
                Ok(None)
            }
        }
    }

    pub fn genblock(&mut self) -> Vec<u8> {
        let b = self.block.clone().into_vec();
        self.block.clear();
        self.cur_header = None;
        self.last_delta = None;
        self.last_entry = None;
        self.value_encoder.reset();
        b
    }

    fn encode_dod(&mut self, dod: i64) {
        let mut neg: bool = false;
        let mut encode = 0;
        if dod == 0 {
            self.block.push(false);
            return;
        }
        if TINNY_DTS_RANGE.contains(&dod) {
            encode = 7;
            self.block.extend_from_bitslice(TINNY_DTS.as_bitslice());
            if dod == 64 {
                self.block.extend_from_bitslice(ENCODED_64_7.as_bitslice());
                return;
            }
        } else if SMALL_DTS_RANGE.contains(&dod) {
            encode = 9;
            self.block.extend_from_bitslice(SMALL_DTS.as_bitslice());
            if dod == 256 {
                self.block.extend_from_bitslice(ENCODED_256_9.as_bitslice());
                return;
            }
        } else if MEDIUM_DTS_RANGE.contains(&dod) {
            encode = 12;
            self.block.extend_from_bitslice(MEDIUM_DTS.as_bitslice());
            if dod == 2048 {
                self.block
                    .extend_from_bitslice(ENCODED_2048_12.as_bitslice());
                return;
            }
        } else {
            self.block.extend_from_bitslice(LARGE_DTS.as_bitslice());
            let d = dod as i32; // Following the paper specification here.
            self.block.extend_from_raw_slice(&d.to_be_bytes());
        }

        if encode != 0 {
            let mut res = bitvec![Msb0, u8;];
            let mut en = dod;
            if dod < 0 {
                neg = true;
                en *= -1;
            }
            // encode-1 garanties the first bit was used to mark negative integers
            for _i in 0..encode - 1 {
                res.push((en & 1) != 0); //Little trick inspired by https://stackoverflow.com/a/8458376
                en /= 2;
            }
            res.push(neg);
            res.reverse();
            self.block.append(&mut res);
        }
    }
}
