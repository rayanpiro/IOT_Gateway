use super::tag::{TagResponse, TagValue};
use async_trait::async_trait;
use std::fmt::Debug;
use serde::Serialize;

#[async_trait]
pub trait THardDevice<C, T> {
    fn new(connection: C) -> Self;
    fn get_device_name(&self) -> String;
    async fn read(&self, tag: &T) -> Result<TagResponse, ReadError>;
    async fn write(&self, tag: &T, value: TagValue) -> Result<(), WriteError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct WriteError(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct ReadError(pub String);
