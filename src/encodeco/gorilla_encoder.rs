use super::value_encoder::ValueEncoder;
use crate::{errors::RstzError, events::LogEvent};
use bitvec::prelude::*;


pub struct GorillaEncoder {
    last_value: Option<f64>,
    last_xor: Option<u64>,
    block: BitVec<Msb0, u8>,
}

impl ValueEncoder for GorillaEncoder {
    fn new() -> Self {
        GorillaEncoder {
            last_value: None,
            last_xor: None,
            block: BitVec::new(),
        }
    }

    fn reset(&mut self) {
        self.last_value = None;
        self.last_xor = None;
        self.block.clear();
    }

    fn compress(
        &mut self,
        field: &str,
        entry: &LogEvent,
    ) -> Result<Option<&BitSlice<Msb0, u8>>, RstzError> {
        let field_value = entry.get_value(&field).ok_or(RstzError::from_none())?;
        let num = field_value
            .as_f64()
            .ok_or(RstzError::new("Cannot represent JSON Value as f64."))?;
        
        if let Some(last_value) = self.last_value {
            self.block.clear();
            let xor = fxor(last_value, num).to_bits();
            if xor == 0 {
                self.block.push(false);
                self.last_value = Some(num);
                self.last_xor = Some(xor);
                return Ok(Some(&self.block.as_bitslice()));
            }

            if self.last_xor.is_some()
                && xor.leading_zeros() >= self.last_xor.unwrap().leading_zeros()
                && xor.trailing_zeros() >= self.last_xor.unwrap().trailing_zeros()
            {
                let mut aux = xor;
                for _i in 0..64 {
                    self.block.push((aux & 1) != 0);
                    aux /= 2;
                }
                self.block
                    .drain(((64 - self.last_xor.unwrap().leading_zeros()) as usize)..64);
                self.block
                    .drain(0..(self.last_xor.unwrap().trailing_zeros() as usize));
                self.block.push(false);
            } else {
                let mut aux = xor;
                let mut zeros = xor.leading_zeros();
                let mut signif = 64 - xor.trailing_zeros() - xor.leading_zeros();
                for _i in 0..64 {
                    self.block.push((aux & 1) != 0);
                    aux /= 2;
                }
                self.block.drain(((64 - xor.leading_zeros()) as usize)..64);
                self.block.drain(0..(xor.trailing_zeros() as usize));
                for _i in 0..6 {
                    self.block.push((signif & 1) != 0);
                    signif /= 2;
                }
                for _i in 0..5 {
                    self.block.push((zeros & 1) != 0);
                    zeros /= 2;
                }
                self.block.push(true);
            }
            self.block.push(true);
            self.block.reverse();
            self.last_xor = Some(xor);
        } else {
            self.block.extend_from_raw_slice(&num.to_be_bytes());
        }
        self.last_value = Some(num);
        Ok(Some(self.block.as_bitslice()))
    }
}

fn fxor(last_value: f64, value: f64) -> f64 {
    let lv_bytes = last_value.to_bits();
    let v_bytes = value.to_bits();
    return f64::from_bits(lv_bytes ^ v_bytes);
}
