use super::tag::{TagId, TagResponse, TagValue, Tag};
use async_trait::async_trait;

#[async_trait]
pub trait Device {
    type TagType: Tag;
    async fn read(&self, tag: &Self::TagType) -> Result<TagResponse, ReadError>;
    async fn write(&self, tag: &Self::TagType, value: TagValue) -> Result<(), WriteError>;
}

#[derive(Debug, Clone)]
pub struct WriteError(pub String);

#[derive(Debug, Clone)]
pub struct ReadError(pub String);