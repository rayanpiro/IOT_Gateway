use serde::Serialize;
use std::{marker::PhantomData, str::FromStr};

use super::device::{ReadError, THardDevice, WriteError};
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

#[derive(Debug, Clone)]
pub enum TagReadFrequency {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
}

impl TagReadFrequency {
    pub fn to_seconds(&self) -> u64 {
        match self {
            Self::Seconds(sec) => *sec,
            Self::Minutes(min) => min * 60,
            Self::Hours(hour) => hour * 3600,
        }
    }
}

impl FromStr for TagReadFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted: Vec<_> = s.split(' ').collect();
        let ammount = splitted.first().unwrap().parse().unwrap();
        let marker = splitted.get(1).unwrap();
        match *marker {
            "s" => Ok(Self::Seconds(ammount)),
            "m" => Ok(Self::Minutes(ammount)),
            "h" => Ok(Self::Hours(ammount)),
            _ => unimplemented!("Invalid marker!"),
        }
    }
}

#[async_trait]
pub trait TTag: Send + Sync {
    async fn read(&self) -> Result<TagResponse, ReadError>;
    async fn write(&self, value: TagValue) -> Result<(), WriteError>;
    fn get_tag(&self) -> Arc<dyn TValidTag>;
    fn get_device_name(&self) -> &str;
}

pub trait TValidTag {
    fn get_name(&self) -> &str;
    fn get_freq(&self) -> &TagReadFrequency;
}

use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Debug, Clone)]
pub struct TagId<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: TValidTag + Send + Sync> {
    pub handler: Arc<Mutex<T>>,
    pub tag: Arc<S>,
    pub device_name: String,
    pub _phantom: PhantomData<C>,
}

#[async_trait]
impl<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: TValidTag + Send + Sync + 'static> TTag
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

    fn get_tag(&self) -> Arc<dyn TValidTag> {
        self.tag.clone()
    }

    fn get_device_name(&self) -> &str {
        &self.device_name
    }
}
