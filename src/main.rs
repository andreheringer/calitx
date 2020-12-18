mod errors;
mod log_event;
mod stream;
mod tree;

fn main() {
    let u = stream::read_log_events_from_file("test.json").unwrap();
    println!("{:#?}", u);
}
