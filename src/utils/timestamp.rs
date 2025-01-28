use chrono::{NaiveDateTime, Utc};

pub fn convert_timestamp_to_utc(timestamp: i64) -> chrono::DateTime<Utc> {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0); // 0 represents the nanoseconds part
    let datetime_utc = naive_datetime.and_utc();
    datetime_utc
}
