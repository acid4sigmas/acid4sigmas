use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_uid() -> i64 {
    let epoch = 1_704_037_200_000;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let timestamp = now - epoch;

    let timestamp_part = (timestamp & 0x3FFFFFFFFFF) << 22;

    let machine_id = rand::thread_rng().gen_range(0..1024);
    let machine_id_part = (machine_id & 0x3FF) << 12;

    let sequence = rand::thread_rng().gen_range(0..4096);

    let uid = timestamp_part | machine_id_part | sequence;
    uid as i64
}
