use super::device::{Device, ReadError};

pub trait Tag {}

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

pub struct TagId<T: Device> {
    pub tag: <T as Device>::TagType,
    pub handler: T,
}

impl<T: Device> TagId<T> {
    pub async fn read(&self) -> Result<TagResponse, ReadError> {
        let tag = &self.tag;
        self.handler.read(tag).await
    }
}