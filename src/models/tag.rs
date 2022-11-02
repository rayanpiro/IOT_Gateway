use std::{marker::PhantomData, str::FromStr};

use super::device::{ReadError, WriteError, THardDevice};
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

#[derive(Debug, Clone)]
pub enum TagReadFrequency {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
}

impl TagReadFrequency {
    pub fn to_seconds(&self) -> u64 {
        match self {
            Self::Seconds(sec) => sec.clone(),
            Self::Minutes(min) => min*60,
            Self::Hours(hour) => hour*3600,
        }
    }
}

impl FromStr for TagReadFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted: Vec<_> = s.split(" ").collect();
        let ammount = splitted.get(0).unwrap().parse().unwrap();
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
}

pub trait TValidTag {
    fn get_name(&self) -> &str;
    fn get_freq(&self) -> &TagReadFrequency;
}

use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct TagId<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: TValidTag + Send + Sync> {
    pub handler: Arc<T>,
    pub tag: Arc<S>,
    pub _phantom: PhantomData<C>,
}

#[async_trait]
impl<T: THardDevice<C, S> + Send + Sync, C: Send + Sync, S: TValidTag + Send + Sync + 'static> TTag for TagId<T, C, S> {
    async fn read(&self) -> Result<TagResponse, ReadError> {
        self.handler.read(&self.tag).await
    }

    async fn write(&self, value: TagValue) -> Result<(), WriteError> {
        self.handler.write(&self.tag, value).await
    }

    fn get_tag(&self) -> Arc<dyn TValidTag> {
        self.tag.clone()
    }

}
