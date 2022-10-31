use std::marker::PhantomData;

use super::device::{ReadError, THardDevice};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct TagResponse {
    pub id: String,
    pub value: TagValue,
}

#[derive(Debug, Clone)]
pub enum TagValue {
    F32(f32),
    // U32(u32),
    I32(i32),
    // String(String),
}

// pub enum TagReadFrequency {
//     Miliseconds(u32),
//     Seconds(u32),
//     Minutes(u32),
//     Hours(u32),
// }

#[async_trait]
pub trait TTag {
    async fn read(&self) -> Result<TagResponse, ReadError>;
}

#[derive(Debug, Clone)]
pub struct TagId<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: Send + Sync> {
    pub handler: T,
    pub tag: S,
    pub _phantom: PhantomData<C>,
}

#[async_trait]
impl<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: Send + Sync> TTag for TagId<T, C, S> {
    async fn read(&self) -> Result<TagResponse, ReadError> {
        self.handler.read(&self.tag).await
    }
}
