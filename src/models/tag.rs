use serde::Serialize;
use std::marker::PhantomData;

use super::device::{ReadError, ReadFrequency, THardDevice, WriteError};
use async_trait::async_trait;

const TAG_REQUEST_SECONDS_TO_TIMEOUT: u64 = 4;

#[derive(Debug, Clone, Serialize)]
pub struct TagResponse {
    pub id: String,
    pub value: TagValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TagValue {
    F32(f32),
    // U32(u32),
    I32(i32),
    // String(String),
}

impl ToString for TagValue {
    fn to_string(&self) -> String {
        match self {
            Self::I32(x) => x.to_string(),
            Self::F32(x) => x.to_string(),
        }
    }
}

#[async_trait]
pub trait TTag: Send + Sync {
    async fn read(&self) -> Result<TagResponse, ReadError>;
    async fn write(&self, value: TagValue) -> Result<(), WriteError>;
    fn tag(&self) -> Arc<dyn Named>;
    async fn freq(&self) -> ReadFrequency;
    fn device_name(&self) -> &str;
}

pub trait Named {
    fn name(&self) -> &str;
}

use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Debug, Clone)]
pub struct TagId<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: Named + Send + Sync> {
    pub handler: Arc<Mutex<T>>,
    pub tag: Arc<S>,
    pub device_name: String,
    pub _phantom: PhantomData<C>,
}

#[async_trait]
impl<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: Named + Send + Sync + 'static> TTag
    for TagId<T, C, S>
{
    async fn read(&self) -> Result<TagResponse, ReadError> {
        let device_lock = self.handler.lock().await;
        tokio::time::timeout(
            tokio::time::Duration::new(TAG_REQUEST_SECONDS_TO_TIMEOUT, 0),
            device_lock.read(&self.tag),
        )
        .await
        .map_err(|err| ReadError(err.to_string()))?
    }

    async fn write(&self, value: TagValue) -> Result<(), WriteError> {
        let device_lock = self.handler.lock().await;
        tokio::time::timeout(
            tokio::time::Duration::new(TAG_REQUEST_SECONDS_TO_TIMEOUT, 0),
            device_lock.write(&self.tag, value),
        )
        .await
        .map_err(|err| WriteError(err.to_string()))?
    }

    fn tag(&self) -> Arc<dyn Named> {
        self.tag.clone()
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }

    async fn freq(&self) -> ReadFrequency {
        self.handler.lock().await.get_freq().to_owned()
    }
}
