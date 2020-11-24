extern crate time;
use time::{Time, date, time, Date};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    let t = date!(2020-11-11);
    let x = date!(2020-11-11);

    let mut hasher = DefaultHasher::new();
    let ht = t.hash(&mut hasher);
    print!("T hashed is: {:x}\n", hasher.finish());
    let hx = x.hash(&mut hasher);
    print!("X hashed is: {:x}\n", hasher.finish());
}
