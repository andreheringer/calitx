extern crate bitvec;
extern crate chrono;

use crate::compress::Gorilla;
use crate::timeseries::{DateDataPoint, TimeSerie};
use bitvec::prelude::*;
use chrono::{Duration, NaiveDateTime};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

impl Gorilla for File {
    fn compress(&self, output_file_path: String, interval: i64) -> Result<usize, Box<dyn Error>> {
        let reader = BufReader::new(self);
        let written = 0;
        let datapoints: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
        let interval_duration = Duration::seconds(interval);
        let timeseries: TimeSerie = TimeSerie::new(&datapoints, interval_duration)?;
        let mut output = File::create(output_file_path)?;
        for batch in timeseries.batches {
            let compressed = compress_batch(batch.points, batch.header)?;
            written += output.write(compressed.as_slice())?;
        }
        Ok(written)
    }
}

fn compress_batch(
    points: &[DateDataPoint],
    header: NaiveDateTime,
) -> Result<BitVec<Msb0, u8>, Box<dyn Error>> {
    let mut last_delta: i64 = 0;
    let mut last_value: f64 = 0.0;

    let mut batch = BitVec::<Msb0, u8>::from_slice(&header.timestamp().to_be_bytes());

    for i in 0..points.len() {
        if i == 0 {
            let delta = points[0].time.signed_duration_since(header).num_seconds();
            batch.append(&mut BitVec::<Msb0, u8>::from_slice(
                &(delta as i16).to_be_bytes(),
            ));
            let value = points[0].value;
            batch.append(&mut BitVec::<Msb0, u8>::from_slice(&value.to_be_bytes()));
            last_delta = delta;
            last_value = floating_xor(0.0, points[0].value);
        } else {
            let delta = points[i]
                .time
                .signed_duration_since(points[i - 1].time)
                .num_seconds();
            let value = floating_xor(points[i - 1].value, points[i].value);
            append_delta(&mut batch, delta - last_delta);
            append_value(&mut batch, last_value, value);
            last_delta = delta;
            last_value = value;
        }
    }
    Ok(batch)
}

fn append_delta(vec: &mut BitVec<Msb0, u8>, delta: i64) {
    if delta == 0 {
        vec.push(false);
    } else if delta >= -63 && delta <= 64 {
        vec.append(&mut bitvec![Msb0, u8; 1, 0]);
        vec.append(&mut tsdelta_encode(delta, 7));
    } else if delta >= -255 && delta <= 256 {
        vec.append(&mut bitvec![Msb0, u8; 1, 1, 0]);
        vec.append(&mut tsdelta_encode(delta, 9));
    } else if delta >= -2047 && delta <= 2048 {
        vec.append(&mut bitvec![Msb0, u8; 1, 1, 1, 0]);
        vec.append(&mut tsdelta_encode(delta, 12));
    } else {
        vec.append(&mut bitvec![Msb0, u8; 1, 1, 1, 1]);
        let d = delta as i32; // Following the paper specification here.
        vec.append(&mut BitVec::<Msb0, u8>::from_slice(&d.to_be_bytes()));
    }
}

fn tsdelta_encode(src: i64, encode: u8) -> BitVec<Msb0, u8> {
    let mut res = bitvec![Msb0, u8;];
    let mut neg: bool = false;
    let mut en = src;
    if src == 64 && encode == 7 {
        res.append(&mut bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0]);
        return res;
    }

    if src == 256 && encode == 9 {
        res.append(&mut bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        return res;
    }

    if src == 2048 && encode == 12 {
        res.append(&mut bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        return res;
    }

    if src < 0 {
        neg = true;
        en *= -1;
    }
    // encode-1 garanties the first bit was used to mark negative integers
    for i in 0..encode - 1 {
        res.push((en & 1) != 0);
        en /= 2;
    }
    res.push(neg);
    res.reverse();
    res
}

fn append_value(vec: &mut BitVec<Msb0, u8>, last_xor: f64, xor: f64) {
    if xor == 0.0 {
        vec.push(false);
        return;
    }
    vec.push(true);
    let leading = xor.to_bits().leading_zeros();
    let trailling = xor.to_bits().trailing_zeros();
    let last_leading = last_xor.to_bits().leading_zeros();
    let last_trailling = last_xor.to_bits().trailing_zeros();
    if leading >= last_leading && trailling >= last_trailling {
        vec.push(false);
        vec.append(&mut bitsvalue_encode(
            xor.to_bits(),
            last_xor.to_bits(),
            true,
        ));
    } else {
        vec.push(true);
        vec.append(&mut bitsvalue_encode(
            xor.to_bits(),
            last_xor.to_bits(),
            false,
        ));
    }
}

fn bitsvalue_encode(src: u64, last: u64, fit: bool) -> BitVec<Msb0, u8> {
    let mut res = bitvec![Msb0, u8;];
    let mut aux = src;
    if fit {
        for i in 0..(64 - last.trailing_zeros()) {
            if i < last.leading_zeros() {
                continue;
            }
            res.push((aux & 1) != 0);
            aux /= 2;
        }
    } else {
        let mut zeros = src.trailing_zeros();
        let mut signif = 64 - src.trailing_zeros() - src.leading_zeros();
        for i in 0..5 {
            res.push((zeros & 1) != 0);
            zeros /= 2;
        }
        for i in 0..6 {
            res.push((signif & 1) != 0);
            signif /= 2;
        }
        for i in 0..(64 - src.trailing_zeros()) {
            if i < src.leading_zeros() {
                continue;
            }
            res.push((aux & 1) != 0);
            aux /= 2;
        }
    }
    res.reverse();
    res
}

fn floating_xor(last_value: f64, value: f64) -> f64 {
    let lv_bytes = last_value.to_bits();
    let v_bytes = value.to_bits();
    f64::from_bits(lv_bytes ^ v_bytes)
}
