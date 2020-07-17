extern crate bitstream_io;
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
        let mut size = 3;
        for byte in buf {
            if *byte != 0 {
                size -= 1;
            }
        }
        if let Some(byte) = self.buff_datetime_size(size) {
            ready.push(byte);
        }
        for byte in buf {
            if *byte != 0 {
                ready.push(self.buff_byte(*byte));
            }
        }
        ready
    }

    fn buff_datetime_size(&mut self, size: u8) -> Option<u8> {
        let mut res: Option<u8> = None;
        for i in (0..size).rev() {
            if self.used_bits == 8 {
                res = Some(self.buffer);
                self.buffer = 0;
                self.used_bits = 0;
            }
            if i == 0 {
                self.buffer = self.buffer << 1;
                self.used_bits += 1;
            } else {
                self.buffer = (self.buffer << 1) | 1;
                self.used_bits += 1;
            }
        }
        res
    }

    fn buff_byte(&mut self, byte: u8) -> u8 {
        let left = byte >> self.used_bits;
        let aux = byte << self.used_bits;
        let right = aux >> self.used_bits;
        let res = self.buffer | left;
        self.buffer = right;
        res
    }
}

pub fn compress<'dp>(
    raw_data_points: &'dp Vec<DateDataPoint>,
    time_batch_interval: Duration,
    output_file_name: String,
) -> Result<usize, Box<dyn Error>> {
    let mut writen: usize = 0;
    let timeseries_batches: TimeSerie = TimeSerie::new(raw_data_points, time_batch_interval)?;
    let mut header = timeseries_batches.start_date.clone();

    let mut output_file = File::create(output_file_name)?;

    for batch in &timeseries_batches.data_points {
        if let Some(batch_content) = batch {
            writen += compress_batch(batch_content, header, &mut output_file)?;
            print!("{:?}", writen);
        }
        header = header.checked_add_signed(time_batch_interval).unwrap();
    }
    Ok(writen)
}

fn compress_batch(
    data_points: &[DateDataPoint],
    header: NaiveDateTime,
    output_file: &mut File,
) -> Result<usize, Box<dyn Error>> {
    let mut bytes_written: usize = 0;
    let mut last_delta: i16 = 0;
    let header_value: i64 = header.timestamp();
    let header_bytes = header_value.to_be_bytes();

    bytes_written += output_file.write(&header_bytes)?;
    let mut bit_buffer = BitBuffer::new();

    for i in 0..data_points.len() {
        if i == 0 {
            // Reservando 16 bits ao inves de 14 para o primeiro delta por motivos de simplicidade de código.
            // Isso faz com que o delta máximo do batch seja de 9 horas, ao contrário das 2 horas do
            // algoritimo original.
            let delta = data_points[0]
                .date_time
                .signed_duration_since(header)
                .num_seconds() as i16;
            let delta_bytes = delta.to_be_bytes();
            bytes_written += output_file.write(&delta_bytes)?;
            let first_value_bytes = data_points[0].passenger_count.to_be_bytes();
            bytes_written += output_file.write(&first_value_bytes)?;
            last_delta = delta;
        } else {
            let delta = data_points[i]
                .date_time
                .signed_duration_since(data_points[i - 1].date_time)
                .num_seconds() as i16;
            let delta_of_deltas = delta - last_delta;
            let delta_of_deltas_bytes = delta_of_deltas.to_be_bytes();
            let ready = bit_buffer.buff_datetime(&delta_of_deltas_bytes);
            for byte in ready {
                bytes_written += output_file.write(&[byte])?;
            }
            last_delta = delta;
        }
    }

    Ok(bytes_written)
}
