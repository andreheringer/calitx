use std::fmt;

#[derive(Debug)]
pub struct Delta {
    source_size: usize,
    target_size: usize,
    pub ops: Vec<Op>,
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
        write!(f, "Size (Source: {} Target: {}), Ops: {:?}", self.source_size, self.target_size, self.ops)
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
            Op::Insert(values) => write!(f, "Insert {:?}", values)
        }
    }
}
