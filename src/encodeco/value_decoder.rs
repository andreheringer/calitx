use serde_json::Value;

pub trait ValueDecoder {
    fn new() -> Self;
    fn decompress() -> Value;
}
