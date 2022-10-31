use super::tag::{TagResponse, TagValue};
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait THardDevice<C, T> {
    fn new(connection: C) -> Self;
    async fn read(&self, tag: &T) -> Result<TagResponse, ReadError>;
    async fn write(&self, tag: &T, value: TagValue) -> Result<(), WriteError>;
}

#[derive(Debug, Clone)]
pub struct WriteError(pub String);

#[derive(Debug, Clone)]
pub struct ReadError(pub String);
