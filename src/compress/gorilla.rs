extern crate bitvec;
extern crate chrono;

use crate::compress::Gorilla;
use crate::timeseries::{DateDataPoint, TimeSerie};
use bitvec::prelude::*;
use chrono::{Duration, NaiveDateTime};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

impl Gorilla for File {
    fn compress(&self, output_file_path: String, interval: i64) -> Result<usize, Box<dyn Error>> {
        let reader = BufReader::new(self);
        let datapoints: Vec<DateDataPoint> = serde_json::from_reader(reader)?;
        let interval_duration = Duration::seconds(interval);
        let timeseries: TimeSerie = TimeSerie::new(&datapoints, interval_duration)?;
        let mut output = File::create(output_file_path)?;
        Ok(0)
    }
}

fn compress_batch(
    points: &[DateDataPoint],
    header: NaiveDateTime,
) -> Result<BitVec<Local, u8>, Box<dyn Error>> {
    let mut last_delta: i32 = 0;
    let mut last_value: f32 = 0.0;

    let mut batch = BitVec::<Local, u8>::from_slice(&header.timestamp().to_be_bytes());

    for i in 0..points.len() {
        if i == 0 {
            let delta = points[0].time.signed_duration_since(header).num_seconds() as i32;
            batch.append(&mut BitVec::<Local, u8>::from_slice(&delta.to_be_bytes()));
            let value = points[0].value;
            batch.append(&mut BitVec::<Local, u8>::from_slice(&value.to_be_bytes()));
            last_delta = delta;
            last_value = floating_xor(0.0, points[0].value);
        } else {
            let delta = points[i]
                .time
                .signed_duration_since(points[i - 1].time)
                .num_seconds() as i32;
            let value = floating_xor(points[i -1].value, points[i].value);
            append_delta(&mut batch, delta - last_delta);
            append_value(&mut batch, last_value, value);

        }
    }
    Ok(batch)
}

fn append_delta(vec: &mut BitVec<Local, u8>, delta: i32) {
    if delta == 0 {
        vec.push(false);
    } else if delta >= -63 && delta <= 64 {
        vec.append(&mut bitvec![Local, u8; 1, 0]);
        vec.append(&mut gorilla_encode(delta, 7));
    } else if delta >= -255 && delta <= 256 {
        vec.append(&mut bitvec![Local, u8; 1, 1, 0]);
        vec.append(&mut gorilla_encode(delta, 9));
    } else if delta >= -2047 && delta <= 2048 {
        vec.append(&mut bitvec![Local, u8; 1, 1, 1, 0]);
        vec.append(&mut gorilla_encode(delta, 12));
    } else {
        vec.append(&mut bitvec![Local, u8; 1, 1, 1, 1]);
        vec.append(&mut BitVec::<Local, u8>::from_slice(&delta.to_be_bytes()))
    }
}

fn gorilla_encode(src: i32, encode: u8) -> BitVec<Local, u8> {
    let mut res = bitvec![Local, u8;];
    let mut neg: bool = false;
    let mut en = src;
    if src == 64 && encode == 7 {
        res.append(&mut bitvec![Local, u8; 0, 0, 0, 0, 0, 0, 1]);
        return res;
    }

    if src == 256 && encode == 9 {
        res.append(&mut bitvec![Local, u8; 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        return res;
    }

    if src == 2048 && encode == 12 {
        res.append(&mut bitvec![Local, u8; 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        return res;
    }
    if src < 0 {
        neg = true;
        en *= -1;
    }
    while en != 0 {
        res.push(en % 2 != 0);
        en /= 2;
    }
    res.push(neg);
    res
}

fn append_value(vec: &mut BitVec<Local, u8>, last_xor: f32, xor: f32) {
    if xor == 0.0 {
        vec.push(false);
        return;
    }
    vec.push(true);
    let leading = xor.to_bits().leading_zeros();
    let trailling = xor.to_bits().trailing_zeros();
    let last_leading = last_xor.to_bits().leading_zeros();
    let last_trailling = last_xor.to_bits().trailing_zeros();
    
}

fn floating_xor(last_value: f32, value: f32) -> f32 {
    let lv_bytes = last_value.to_bits();
    let v_bytes = value.to_bits();
    f32::from_bits(lv_bytes ^ v_bytes)
}
