mod errors;
mod log_event;
mod stream;
mod tree;
mod compress;

use compress::xdelta;

fn main() {
//    let u = stream::read_log_events_from_file("test.json").unwrap();
//    println!("{:#?}", u);

    let s = "the quick brown fox jumps over ";
    let t = "a swift auburn fox jumps over three dormant hounds";

    print!("{}", xdelta(&s, &t));
}
