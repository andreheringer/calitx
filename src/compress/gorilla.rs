extern crate bitvec;

use bitvec::prelude::*;

enum GorillaTimeSeriesTable {
    SMALL,
    MEDIUM,
    LARGE,
    HUGE,
}

impl GorillaTimeSeriesTable {
    fn bitmap(&self) -> BitVec {
        match *self {
            GorillaTimeSeriesTable::SMALL => bitvec![1, 0],
            GorillaTimeSeriesTable::MEDIUM => bitvec![1, 1, 0],
            GorillaTimeSeriesTable::LARGE => bitvec![1, 1, 1, 0],
            GorillaTimeSeriesTable::HUGE => bitvec![1, 1, 1, 1],
        }
    }
}


pub fn compress_ts(delta: i64) -> BitVec<Msb0, u8> {
    let mut dod = BitVec::<Msb0, u8>::with_capacity(36);
    if delta == 0 {
        dod.push(false);
    } else if delta >= -63 && delta <= 64 {
        dod.append(&mut GorillaTimeSeriesTable::SMALL.bitmap());
    } else if delta >= -255 && delta <= 256 {
        dod.append(&mut GorillaTimeSeriesTable::MEDIUM.bitmap());
    } else if delta >= -2047 && delta <= 2047 {
        dod.append(&mut GorillaTimeSeriesTable::LARGE.bitmap());
    } else {
        dod.append(&mut GorillaTimeSeriesTable::HUGE.bitmap());
    }
    dod.append(&mut delta_to_bitvec(delta));
    dod
}

fn delta_to_bitvec(d: i64) -> BitVec<Msb0, u8> {
    let mut res = bitvec![Msb0, u8; 0, 1];
    let mut delta = d;
    if delta < 0 {
        res.set(0, true);
        delta *= -1;
    }
    while delta > 0 {
        if delta % 2 == 0 {
            res.push(false);
        } else {
            res.push(true);
        }
        delta /= 2;
    }
    res
}