use crate::errors::RstzError;
use crate::events::LogEvent;
use bitvec::prelude::*;

pub trait ValueEncoder {
    fn new() -> Self;
    fn reset(&mut self);
    fn compress(
        &mut self,
        field: &str,
        entry: &LogEvent,
    ) -> Result<Option<&BitSlice<Msb0, u8>>, RstzError>;
}
