use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime::now panic")
        .as_millis()
}

pub fn gen_request_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}
