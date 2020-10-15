extern crate bitvec;
extern crate chrono;
extern crate serde;

use super::{TimeChunk, TimeSeries, TimedDataPoint};
use bitvec::prelude::*;
use serde::de::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub fn compress_from_file<'de, T: TimedDataPoint + Deserialize<'de> + Clone>(
    input_file: &File,
    output_file: &mut File,
    interval: i64,
) -> Result<usize, Box<dyn Error>> {
    let reader = BufReader::new(input_file);
    let mut written = 0;
    let timeseries = TimeSeries::<BufReader<&File>, T>::from_reader(interval, reader);
    for chunk in timeseries {
        let compressed = compress_chunk(chunk)?;
        written += output_file.write(compressed.as_slice())?;
    }
    Ok(written)
}

fn compress_chunk<T: TimedDataPoint>(
    chunk: TimeChunk<T>,
) -> Result<BitVec<Msb0, u8>, Box<dyn Error>> {
    let mut last_delta: i64 = 0;
    let mut last_value: f64 = 0.0;

    let mut compressed = BitVec::<Msb0, u8>::from_slice(&chunk.header.timestamp().to_be_bytes());

    for i in 0..chunk.points.len() {
        if i == 0 {
            let delta = chunk.points[0]
                .naivetime()
                .signed_duration_since(chunk.header)
                .num_seconds();
            compressed.append(&mut BitVec::<Msb0, u8>::from_slice(
                &(delta as i16).to_be_bytes(),
            ));
            let value = chunk.points[0].value().as_f64().unwrap();
            compressed.append(&mut BitVec::<Msb0, u8>::from_slice(&value.to_be_bytes()));
            last_delta = delta;
            last_value = floating_xor(0.0, chunk.points[0].value().as_f64().unwrap());
        } else {
            let delta = chunk.points[i]
                .naivetime()
                .signed_duration_since(chunk.points[i - 1].naivetime())
                .num_seconds();
            let value = floating_xor(
                chunk.points[i - 1].value().as_f64().unwrap(),
                chunk.points[i].value().as_f64().expect("None or bad formated value"),
            );
            append_delta(&mut compressed, delta - last_delta);
            append_value(&mut compressed, last_value, value);
            last_delta = delta;
            last_value = value;
        }
    }
    Ok(compressed)
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
    for _i in 0..encode - 1 {
        res.push((en & 1) != 0); //Little trick inspired by https://stackoverflow.com/a/8458376
        en /= 2;
    }
    res.push(neg);
    res.reverse();
    return res;
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
        for _i in 0..64 {
            res.push((aux & 1) != 0);
            aux /= 2;
        }
        res.drain(((64 - last.leading_zeros()) as usize)..64);
        res.drain(0..(last.trailing_zeros() as usize));
    } else {
        let mut zeros = src.leading_zeros();
        let mut signif = 64 - src.trailing_zeros() - src.leading_zeros();
        for _i in 0..64 {
            res.push((aux & 1) != 0);
            aux /= 2;
        }
        res.drain(((64 - src.leading_zeros()) as usize)..64);
        res.drain(0..(src.trailing_zeros() as usize));
        for _i in 0..6 {
            res.push((signif & 1) != 0);
            signif /= 2;
        }
        for _i in 0..5 {
            res.push((zeros & 1) != 0);
            zeros /= 2;
        }
    }
    res.reverse();
    return res;
}

fn floating_xor(last_value: f64, value: f64) -> f64 {
    let lv_bytes = last_value.to_bits();
    let v_bytes = value.to_bits();
    return f64::from_bits(lv_bytes ^ v_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_two_f64() {
        let f = floating_xor(123.125f64, 123.125f64);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn bitsvalue_encode_fit() {
        let t = bitsvalue_encode(123456u64, 123460u64, true);
        let sintec = bitvec![Msb0, u8; 1,1,1,1,0,0,0,1,0,0,1,0,0,0,0];
        assert_eq!(t, sintec);
    }

    #[test]
    fn bitsvalue_encode_nofit() {
        let n: f64 = floating_xor(12f64, 24f64);
        let t = bitsvalue_encode(n.to_bits(), 1f64.to_bits(), false);
        let sintec = bitvec![Msb0, u8; 0,1,0,1,1,0,0,0,0,0,1,1];
        assert_eq!(t, sintec);
    }

    #[test]
    fn tsdelta_encode_small_upper() {
        let t = tsdelta_encode(64i64, 7);
        let sintec = bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0];
        assert_eq!(t, sintec);
        let n = tsdelta_encode(24i64, 7);
        let sintec2 = bitvec![Msb0, u8; 0, 0, 1, 1, 0, 0, 0];
        assert_eq!(n, sintec2);
    }

    #[test]
    fn tsdelta_encode_small_lower() {
        let t = tsdelta_encode(-63i64, 7);
        let sintec = bitvec![Msb0, u8; 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(t, sintec);
    }

    #[test]
    fn tsdelta_encode_medium_upper() {
        let t = tsdelta_encode(256i64, 9);
        let sintec = bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(t, sintec);
        let n = tsdelta_encode(72i64, 9);
        let sintec2 = bitvec![Msb0, u8; 0, 0, 1, 0, 0, 1, 0, 0, 0];
        assert_eq!(n, sintec2);
    }

    #[test]
    fn tsdelta_encode_medium_lower() {
        let t = tsdelta_encode(-255i64, 9);
        let sintec = bitvec![Msb0, u8; 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(t, sintec);
    }

    #[test]
    fn tsdelta_encode_large_upper() {
        let t = tsdelta_encode(2048i64, 12);
        let sintec = bitvec![Msb0, u8; 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(t, sintec);
        let n = tsdelta_encode(521i64, 12);
        let sintec2 = bitvec![Msb0, u8; 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1];
        assert_eq!(n, sintec2);
    }

    #[test]
    fn tsdelta_encode_large_lower() {
        let t = tsdelta_encode(-2047i64, 12);
        let sintec = bitvec![Msb0, u8; 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(t, sintec);
    }
}
