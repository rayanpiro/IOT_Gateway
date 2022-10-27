use super::device::Device;

#[derive(Debug, Clone)]
pub struct TagResponse {
    pub id: String,
    pub value: TagValue,
}

#[derive(Debug, Clone)]
pub enum TagValue {
    f32(f32),
    u32(u32),
    i32(i32),
    String(String),
}

pub enum TagReadFrequency {
    Miliseconds(u32),
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u32),
}

use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct TagId {
    pub id: String,
    pub handler: Arc<Mutex<Box<dyn Device+Send>>>,
}