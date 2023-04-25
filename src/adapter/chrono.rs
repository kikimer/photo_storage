use std::time::SystemTime;
use chrono::{Datelike, DateTime, Local};

pub fn year(time: SystemTime) -> i32 {
    let creation_time: DateTime<Local> = time.into();
    creation_time.year()
}