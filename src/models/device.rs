use super::tag::{TagResponse, TagValue};
use async_trait::async_trait;
use serde::Serialize;
use std::{fmt::Debug, str::FromStr};

#[async_trait]
pub trait THardDevice<C, T> {
    fn new(connection: C) -> Self;
    fn get_freq(&self) -> &ReadFrequency;
    fn get_device_name(&self) -> String;
    async fn read(&self, tag: &T) -> Result<TagResponse, ReadError>;
    async fn write(&self, tag: &T, value: TagValue) -> Result<(), WriteError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct WriteError(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct ReadError(pub String);

#[derive(Debug, Clone)]
pub enum ReadFrequency {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
}

impl ReadFrequency {
    pub fn to_seconds(&self) -> u64 {
        match self {
            Self::Seconds(sec) => *sec,
            Self::Minutes(min) => min * 60,
            Self::Hours(hour) => hour * 3600,
        }
    }
}

impl FromStr for ReadFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted: Vec<_> = s.split(' ').collect();
        let ammount = splitted.first().unwrap().parse().unwrap();
        let &marker = splitted.get(1).unwrap();
        match marker {
            "s" => Ok(Self::Seconds(ammount)),
            "m" => Ok(Self::Minutes(ammount)),
            "h" => Ok(Self::Hours(ammount)),
            _ => unimplemented!("Invalid marker!"),
        }
    }
}
