mod delta;

use std::collections::HashMap;
use std::mem;
use delta::{Delta, Op};

const BLOCK_SIZE: usize = 16;
const MAX_COPY_SIZE: usize = 0x00ff_ffff;
const MAX_INSERT_SIZE: usize = 0x7f;

pub fn xdelta(source: &str, target: &str) -> Delta {
    let source_bytes = source.as_bytes();
    let target_bytes = target.as_bytes();
    let mut offset_map: HashMap<Vec<u8>, Vec<usize>>  = HashMap::new();

    for (i, chunk) in source_bytes.chunks_exact(BLOCK_SIZE).enumerate() {
        let entry = offset_map.entry(chunk.to_vec());
        entry.or_default().push(i * BLOCK_SIZE);
    }

    let mut compressor = Compressor::new(source_bytes, &offset_map, target_bytes);
    compressor.compress();

    Delta::new(source_bytes.len(), target_bytes.len(), compressor.ops)
}


struct Compressor<'s, 't> {
    source: &'s [u8],
    target: &'t [u8],
    offset: usize,
    offset_map: &'s HashMap<Vec<u8>, Vec<usize>>,
    insert: Vec<u8>,
    ops: Vec<Op>,
}

impl<'s, 't> Compressor<'s, 't> {
    fn new(source:&'s [u8], offset_map: &'s HashMap<Vec<u8>, Vec<usize>>, target: &'t [u8]) -> Self {
        Compressor {
            source,
            target,
            offset: 0,
            offset_map,
            insert: Vec::new(),
            ops: Vec::new(),
        }
    }

    fn offsets(&self, chunk: &[u8]) -> impl Iterator<Item = &usize> {
        self.offset_map.get(chunk).into_iter().flatten()
    }

    fn compress(&mut self) {
        while self.offset < self.target.len() {
            self.compress_chunk();
        }
        self.flush_insert(0);
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::delta::Op::{Copy, Insert};
    //  0               16               32               48
    //  +----------------+----------------+----------------+
    //  |the quick brown |fox jumps over t|he slow lazy dog|
    //  +----------------+----------------+----------------+

    #[test]
    fn compress_string() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "a swift auburn fox jumps over three dormant hounds";

        let delta = xdelta(source, target);

        assert_eq!(
            delta.ops,
            vec![
                Insert("a swift aubur".into()),
                Copy(14, 19),
                Insert("ree dormant hounds".into())
            ]
        );
    }

    #[test]
    fn compress_incomplete_block() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "he quick brown fox jumps over trees";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Copy(1, 31), Insert("rees".into())]);
    }

    #[test]
    fn compress_at_source_start() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "the quick brown ";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Copy(0, 16)]);
    }

    #[test]
    fn compress_at_source_start_with_right_expansion() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "the quick brown fox hops";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Copy(0, 20), Insert("hops".into())]);
    }

    #[test]
    fn compress_at_source_start_with_left_offset() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "behold the quick brown foal";

        let delta = xdelta(source, target);

        assert_eq!(
            delta.ops,
            vec![Insert("behold ".into()), Copy(0, 18), Insert("al".into())]
        );
    }

    #[test]
    fn compress_at_source_end() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "he slow lazy dog";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Copy(32, 16)]);
    }

    #[test]
    fn compress_at_source_end_with_left_expansion() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "under the slow lazy dog";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Insert("und".into()), Copy(28, 20)]);
    }

    #[test]
    fn compress_at_source_end_with_right_offset() {
        let source = "the quick brown fox jumps over the slow lazy dog";
        let target = "under the slow lazy dog's legs";

        let delta = xdelta(source, target);

        assert_eq!(
            delta.ops,
            vec![Insert("und".into()), Copy(28, 20), Insert("'s legs".into())]
        );
    }

    #[test]
    fn compress_unindexed_bytes() {
        let source = "the quick brown fox";
        let target = "see the quick brown fox";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Insert("see ".into()), Copy(0, 19)]);
    }

    #[test]
    fn do_not_compress_unindexed_bytes() {
        let source = "the quick brown fox";
        let target = "a quick brown fox";

        let delta = xdelta(source, target);

        assert_eq!(delta.ops, vec![Insert("a quick brown fox".into())]);
    }
}