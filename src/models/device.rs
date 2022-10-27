use crate::models::tag::{TagInfo, TagResponse, TagValue, TagWrite};

pub trait Device {
    fn read(tag: TagInfo) -> Result<TagResponse, ReadError>;
    fn write(tag: TagWrite, value: TagValue) -> Result<(), WriteError>;
}

pub struct WriteError(String);
pub struct ReadError(String);