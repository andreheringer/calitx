use super::delta::{Delta, Op};
use std::collections::HashMap;
use std::mem;

const BLOCK_SIZE: usize = 16;
const MAX_COPY_SIZE: usize = u8::MAX as usize;
const MAX_INSERT_SIZE: usize = 0x7f;

pub struct DeltaCompressor<'s, 't> {
    source: &'s [u8],
    target: &'t [u8],
    offset: usize,
    offset_map: HashMap<Vec<u8>, Vec<usize>>,
    insert: Vec<u8>,
    ops: Vec<Op>,
}

impl<'s, 't> DeltaCompressor<'s, 't> {
    pub fn new(source: &'s [u8], target: &'t [u8]) -> Self {
        let mut offset_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        DeltaCompressor::refresh_offsets(&mut offset_map, source);
        DeltaCompressor {
            source,
            target,
            offset: 0,
            offset_map,
            insert: Vec::new(),
            ops: Vec::new(),
        }
    }

    pub fn gendelta(&mut self) -> Delta {
        let dops = mem::replace(&mut self.ops, Vec::new());
        Delta::new(self.source.len(), self.target.len(), dops)
    }

    pub fn compress(&mut self) {
        while self.offset < self.target.len() {
            self.compress_chunk();
        }
        self.flush_insert(0);
    }

    fn offsets(&self, chunk: &[u8]) -> impl Iterator<Item = &usize> {
        self.offset_map.get(chunk).into_iter().flatten()
    }

    fn compress_chunk(&mut self) {
        let (mut m_offset, mut m_size) = self.longest_match();
        if m_size == 0 {
            self.push_insert();
        } else {
            self.expand_match(&mut m_offset, &mut m_size);
            self.flush_insert(0);
            self.ops.push(Op::Copy(m_offset, m_size));
        }
    }

    fn longest_match(&self) -> (usize, usize) {
        let end = self.offset + BLOCK_SIZE;
        if end > self.target.len() {
            return (0, 0);
        }

        let slice = &self.target[self.offset..end];
        let mut m_offset = 0;
        let mut m_size = 0;

        for &pos in self.offsets(slice) {
            let remaining = self.remaining_bytes(pos);
            if remaining <= m_size {
                break;
            }

            let s = self.match_from(pos, remaining);

            if m_size < s - pos {
                m_offset = pos;
                m_size = s - pos;
            }
        }

        (m_offset, m_size)
    }

    fn remaining_bytes(&self, pos: usize) -> usize {
        let s_remaining = self.source.len() - pos;
        let t_remaining = self.target.len() - self.offset;

        let sizes = [s_remaining, t_remaining, MAX_COPY_SIZE];
        *sizes.iter().min().unwrap()
    }

    fn match_from(&self, pos: usize, mut remaining: usize) -> usize {
        let mut s = pos;
        let mut t = self.offset;

        while remaining > 0 && self.source.get(s) == self.target.get(t) {
            s += 1;
            t += 1;
            remaining -= 1;
        }
        s
    }

    fn expand_match(&mut self, m_offset: &mut usize, m_size: &mut usize) {
        while *m_offset > 0 && *m_size < MAX_COPY_SIZE {
            if self.source.get(*m_offset - 1) != self.insert.last() {
                break;
            }
            self.offset -= 1;
            *m_offset -= 1;
            *m_size += 1;
            self.insert.pop();
        }
        self.offset += *m_size;
    }

    fn push_insert(&mut self) {
        self.insert.push(self.target[self.offset]);
        self.offset += 1;
        self.flush_insert(MAX_INSERT_SIZE);
    }

    fn flush_insert(&mut self, size: usize) {
        if self.insert.is_empty() || self.insert.len() < size {
            return;
        }
        let insert = mem::replace(&mut self.insert, Vec::new());
        self.ops.push(Op::Insert(insert));
    }

    fn refresh_offsets(offset_map: &mut HashMap<Vec<u8>, Vec<usize>>, source: &[u8]) {
        for (i, chunk) in source.chunks_exact(BLOCK_SIZE).enumerate() {
            let entry = offset_map.entry(chunk.to_vec());
            entry.or_default().push(i * BLOCK_SIZE);
        }
    }

}
