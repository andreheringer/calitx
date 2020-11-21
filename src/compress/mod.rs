pub mod gorilla;

extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::Duration;
use chrono::NaiveDateTime;
use serde_json::de::{IoRead, StreamDeserializer};
use serde_json::value::Value;

pub trait TimedDataPoint {
    fn naivetime(&self) -> NaiveDateTime;
    fn value(&self) -> Value;
}

#[derive(Debug)]
struct TimeChunk<T>
where
    T: TimedDataPoint,
{
    pub header: NaiveDateTime,
    pub points: Vec<T>,
}

struct TimeSeries<'de, R: std::io::Read, D: serde::de::Deserialize<'de> + TimedDataPoint> {
    stream_desirializer: StreamDeserializer<'de, IoRead<R>, D>,
    interval: Duration,
    iter_header: Option<NaiveDateTime>,
    remender: Option<D>,
}

impl<'de, R, D> TimeSeries<'de, R, D>
where
    R: std::io::Read,
    D: serde::de::Deserialize<'de> + TimedDataPoint,
{
    pub fn from_reader(interval_secs: i64, reader: R) -> Self {
        let interval = Duration::seconds(interval_secs);
        let stream_desirializer = serde_json::Deserializer::from_reader(reader).into_iter::<D>();
        TimeSeries {
            stream_desirializer,
            interval,
            iter_header: None,
            remender: None,
        }
    }
}

impl<'de, R, D> Iterator for TimeSeries<'de, R, D>
where
    R: std::io::Read,
    D: serde::de::Deserialize<'de> + TimedDataPoint + Clone,
{
    type Item = TimeChunk<D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_header == None {
            if let Some(Ok(first)) = self.stream_desirializer.next() {
                self.iter_header = Some(first.naivetime().date().and_hms(0, 0, 0));
                self.remender = Some(first);
            }
        }

        if let Some(remender) = self.remender.clone() {
            let mut header = self.iter_header.unwrap();
            while remender.naivetime().signed_duration_since(header) > self.interval {
                header = header.checked_add_signed(self.interval).unwrap();
            }
            let mut points: Vec<D> = Vec::new();
            points.push(remender);
            loop {
                if let Some(res_point) = self.stream_desirializer.next() {
                    let point = match res_point {
                        Ok(res) => res,
                        Err(e) => panic!("Wrong format input {:?}", e),
                    };
                    if point.naivetime().signed_duration_since(header) <= self.interval {
                        points.push(point);
                    } else {
                        self.remender = Some(point);
                        break;
                    }
                } else {
                    self.remender = None;
                    break;
                }
            }
            return Some(TimeChunk { header, points });
        }
        return None;
    }
}
