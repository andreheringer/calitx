#[cfg(test)]
mod test;

use std::fmt;

const MASK: usize = 0x7f;
const SHIFT: usize = 7;

trait Encode {
    fn enconde(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct Delta {
    source_size: usize,
    target_size: usize,
    ops: Vec<Op>,
}

impl Delta {
    pub fn new(source_size: usize, target_size: usize, ops: Vec<Op>) -> Self {
        Delta {
            source_size,
            target_size,
            ops,
        }
    }
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Size (Source: {} Target: {}), Ops: {:?}",
            self.source_size, self.target_size, self.ops
        )
    }
}

impl Encode for Delta {
    fn enconde(&self) -> Vec<u8> {
        let low_bits = |v: usize| -> u8 { (v & MASK) as u8 };

        let variant_encode = |b: &mut Vec<u8>, v: usize| -> () {
            let mut value = v;
            while value > MASK {
                b.push(0x80 | low_bits(value));
                value >>= SHIFT;
            }
            b.push(low_bits(value));
        };
        let mut buffer = Vec::new();
        variant_encode(&mut buffer, self.source_size);
        variant_encode(&mut buffer, self.target_size);
        for op in &self.ops {
            buffer.extend(op.enconde());
        }
        buffer
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Op {
    Copy(usize, usize),
    Insert(Vec<u8>),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Copy(offset, size) => write!(f, "Copy({}, {})", offset, size),
            Op::Insert(values) => write!(f, "Insert {:?}", values),
        }
    }
}

impl Encode for Op {
    fn enconde(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        match self {
            Op::Copy(offset, size) => {
                let value = (size << 32) | offset;
                buffer.push(0x80);
                for i in 0..7 {
                    let byte = (value >> (8 * i)) & 0xff;
                    if byte > 0 {
                        buffer[0] |= 1 << i;
                        buffer.push(byte as u8);
                    }
                }
                buffer
            }

            Op::Insert(values) => {
                buffer.push(values.len() as u8);
                buffer.extend(values);
                buffer
            }
        }
    }
}

