use super::tag::{TagId, TagResponse, TagValue};
use async_trait::async_trait;

#[async_trait]
pub trait Device {
    async fn read(&self) -> Result<TagResponse, ReadError>;
    async fn write(&self, value: TagValue) -> Result<(), WriteError>;
}

#[derive(Debug, Clone)]
pub struct WriteError(pub String);

#[derive(Debug, Clone)]
pub struct ReadError(pub String);