pub type Timestamp = u64;

pub fn current_timestamp() -> Timestamp {
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    if timestamp < 0 {
        panic!("current timestamp err: {}", timestamp)
    }
    timestamp as Timestamp
}
