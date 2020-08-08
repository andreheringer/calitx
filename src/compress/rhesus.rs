extern crate chrono;

use crate::timeseries::{DateDataPoint, TimeSerie};
use chrono::{Duration, NaiveDateTime};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

struct BitBuffer {
    pub used_bits: usize,
    pub buffer: u8,
}

impl BitBuffer {
    pub fn new() -> BitBuffer {
        BitBuffer {
            used_bits: 0,
            buffer: 0,
        }
    }

    pub fn buff_datetime(&mut self, buf: &[u8]) -> Vec<u8> {
        let mut ready: Vec<u8> = Vec::new();
        let mut size = 4;
        for byte in buf {
            if *byte == 0 {
                size -= 1;
            } else {
                break;
            }
        }
        debug!(
            "## TIME: Buffering bytes: {:?}\tTIME: Size Defined: {:?}",
            buf, size
        );
        debug!(
            "--- BitBuffer state: {:#010b} Used bits: {:?} ---",
            self.buffer, self.used_bits
        );
        if let Some(byte) = self.buff_datetime_size(size) {
            ready.push(byte);
        }
        debug!(
            "--- BitBuffer state: {:#010b} Used bits: {:?} ---",
            self.buffer, self.used_bits
        );
        if let Some(res) = self.is_buffer_full() {
            ready.push(res);
        }
        for byte in buf {
            if *byte != 0 {
                ready.push(self.buff_byte(*byte));
            }
        }
        debug!(
            "## TIME: Bytes ready for write: {:?} BitBuffer state: {:#010b} Used bits: {:?}",
            ready, self.buffer, self.used_bits
        );
        ready
    }

    pub fn buff_value(&mut self, last_xor_value: f32, xor_value: f32) -> Vec<u8> {
        let mut ready: Vec<u8> = Vec::new();
        debug!("VALUE: Buffering bytes: {:?}", xor_value);
        debug!(
            "--- BitBuffer state: {:#010b} Used bits: {:?} ---",
            self.buffer, self.used_bits
        );
        if let Some(res) = self.is_buffer_full() {
            ready.push(res);
        }
        self.buffer = self.buffer << 1;
        self.used_bits += 1;

        if xor_value != 0.0 {
            self.buffer |= 1;
            if let Some(res) = self.is_buffer_full() {
                ready.push(res);
            }
            self.buffer = self.buffer << 1;
            self.used_bits += 1;
            let mut bits = xor_value.to_bits();

            let old_range = last_xor_value.to_bits().leading_zeros()
                ..(32 - last_xor_value.to_bits().trailing_zeros());

            if old_range.contains(&xor_value.to_bits().leading_zeros())
                && old_range.contains(&xor_value.to_bits().trailing_zeros())
            {
                bits <<= last_xor_value.to_bits().leading_zeros();
            } else {
                self.buffer |= 1;
                let new_leading_zeros: u8 = xor_value.to_bits().leading_zeros() as u8;
                let new_significant_size: u8 =
                    (32 - xor_value.to_bits().leading_zeros()
                        - xor_value.to_bits().trailing_zeros()) as u8;
                ready.push(new_leading_zeros.to_be_bytes()[0]);
                ready.push(new_significant_size.to_be_bytes()[0]);
                bits <<= new_leading_zeros;
            }

            for byte in &bits.to_be_bytes() {
                if *byte != 0 {
                    ready.push(self.buff_byte(*byte));
                }
            }
        }
        debug!(
            "--- BitBuffer state: {:#010b} Used bits: {:?} ---",
            self.buffer, self.used_bits
        );
        ready
    }

    fn buff_datetime_size(&mut self, size: u8) -> Option<u8> {
        let mut res: Option<u8>;
        if size < 4 && size > 0 {
            for _i in (0..size).rev() {
                res = self.is_buffer_full();
                self.buffer = (self.buffer << 1) | 1;
                self.used_bits += 1;
            }
        }
        res = self.is_buffer_full();
        self.buffer = self.buffer << 1;
        self.used_bits += 1;
        if size == 4 {
            self.buffer |= 1;
            for _i in 0..3 {
                res = self.is_buffer_full();
                self.buffer = (self.buffer << 1) | 1;
                self.used_bits += 1;
            }
        }
        res
    }

    fn buff_byte(&mut self, byte: u8) -> u8 {
        debug!(
            "Buffering: {:#010b} into {:#010b} with used bits: {:?}",
            byte, self.buffer, self.used_bits
        );
        if self.used_bits == 0 {
            return byte;
        }
        let left = byte >> self.used_bits;
        let aux = byte << self.used_bits;
        let right = aux >> self.used_bits;
        let res = (self.buffer << self.used_bits) | left;
        self.buffer = right;
        debug!(
            "Ready for write: {:#010b} BitBuffer State {:#010b}",
            res, self.buffer
        );
        res
    }

    fn floating_xor(last_value: f32, value: f32) -> f32 {
        let lv_bytes = last_value.to_bits();
        let v_bytes = value.to_bits();
        f32::from_bits(lv_bytes ^ v_bytes)
    }

    fn is_buffer_full(&mut self) -> Option<u8> {
        let res: Option<u8>;
        if self.used_bits == 8 {
            res = Some(self.buffer);
            self.buffer = 0;
            self.used_bits = 0;
        } else {
            res = None;
        }
        res
    }
}

pub fn compress<'dp>(
    raw_data_points: &'dp Vec<DateDataPoint>,
    time_batch_interval: Duration,
    output_file_name: String,
) -> Result<usize, Box<dyn Error>> {
    let mut written: usize = 0;
    let timeseries: TimeSerie = TimeSerie::new(raw_data_points, time_batch_interval)?;
    let mut output_file = File::create(output_file_name)?;
    for batch in timeseries.batches {
        written += compress_batch(batch.points, batch.header, &mut output_file)?;
    }
    Ok(written)
}

fn compress_batch(
    points: &[DateDataPoint],
    header: NaiveDateTime,
    output_file: &mut File,
) -> Result<usize, Box<dyn Error>> {
    let mut bytes_written: usize = 0;
    let mut last_delta: i32 = 0;
    let mut last_xor_value = 0.0;
    debug!("#### Header Value: {:?} ####", header);
    let header_value: i64 = header.timestamp();
    let header_bytes = header_value.to_be_bytes();

    bytes_written += output_file.write(&header_bytes)?;
    debug!("Bytes Writen: {:?}", bytes_written);
    let mut bit_buffer = BitBuffer::new();
    debug!("Bit buffer inittiated...");

    for i in 0..points.len() {
        if i == 0 {
            // Using 32 bits for the first delta for simplicity
            let delta = points[0].time.signed_duration_since(header).num_seconds() as i32;
            let delta_bytes = delta.to_be_bytes();
            debug!(
                "First Delta: {:?} \t First Delta Bytes: {:?}",
                delta, delta_bytes
            );
            bytes_written += output_file.write(&delta_bytes)?;
            debug!("Bytes Writen: {:?}", bytes_written);

            let first_value_bytes = points[0].value.to_be_bytes();
            debug!(
                "First Value {:?} \t First Value Bytes: {:?}",
                points[0].value, first_value_bytes
            );
            bytes_written += output_file.write(&first_value_bytes)?;
            debug!("Bytes Writen: {:?}", bytes_written);

            last_delta = delta;
            last_xor_value = BitBuffer::floating_xor(0.0, points[0].value);
        } else {
            let delta = points[i]
                .time
                .signed_duration_since(points[i - 1].time)
                .num_seconds() as i32;
            debug!(
                "Delta: {:?} \t Last Delta: {:?} \t Delta of Deltas: {:?}",
                delta,
                last_delta,
                delta - last_delta
            );
            let mut ready: Vec<u8> = bit_buffer.buff_datetime(&(delta - last_delta).to_be_bytes());
            //TODO: Should probably optimaze this later, here I'm calling write multiple times and alocating way to much memory
            for byte in ready {
                bytes_written += output_file.write(&[byte])?;
            }
            debug!("Bytes Writen: {:?}", bytes_written);
            let xor_value = BitBuffer::floating_xor(points[i - 1].value, points[i].value);
            ready = bit_buffer.buff_value(last_xor_value, xor_value);
            // TODO: Same as above
            for byte in ready {
                bytes_written += output_file.write(&[byte])?;
            }
            debug!("Bytes Writen: {:?}", bytes_written);
            last_delta = delta;
            last_xor_value = xor_value;
        }
    }

    Ok(bytes_written)
}
