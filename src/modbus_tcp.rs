use crate::{gen_matcher, gen_readable_struct};
use tokio_modbus::{client::Context, prelude::*};

gen_matcher!(
    enum Swap {
        BigEndian,
        LittleEndian,
        BigEndianSwap,
        LittleEndianSwap,
    }
);

gen_matcher!(
    enum Type {
        Integer,
        Float,
    }
);

gen_matcher!(
    enum Command {
        Coil,
        Discrete,
        Holding,
        Input,
    }
);

gen_readable_struct!(
    struct ModbusTcpConnection {
        name: String,
        ip: std::net::IpAddr,
        port: u16,
        slave: u8,
    }
);

gen_readable_struct!(
    struct ModbusTcpTag {
        name: String,
        address: u16,
        length: u16,
        command: Command,
        swap: Swap,
        data_type: Type,
        read_freq: TagReadFrequency,
        multiplier: f32,
    }
);

impl TValidTag for ModbusTcpTag {
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_freq(&self) -> &TagReadFrequency {
        &self.read_freq
    }
}

#[derive(Debug, Clone)]
pub struct ModbusTcpDevice(pub ModbusTcpConnection);

use crate::models::device::{ReadError, THardDevice, WriteError};
use crate::models::tag::{TValidTag, TagReadFrequency, TagResponse, TagValue};

#[derive(Debug, Clone)]
struct ModbusTcpError(String);

use std::net::SocketAddr;

impl ModbusTcpDevice {
    async fn connect(&self) -> Result<Context, ModbusTcpError> {
        let socket_address = SocketAddr::new(self.0.ip, self.0.port);

        match client::tcp::connect_slave(socket_address, Slave(self.0.slave)).await {
            Ok(ctx) => Ok(ctx),
            Err(err) => Err(ModbusTcpError(err.to_string())),
        }
    }
}

use async_trait::async_trait;

#[async_trait]
impl THardDevice<ModbusTcpConnection, ModbusTcpTag> for ModbusTcpDevice {
    fn new(connection: ModbusTcpConnection) -> Self {
        ModbusTcpDevice(connection)
    }

    async fn read(&self, tag: &ModbusTcpTag) -> Result<TagResponse, ReadError> {
        let mut ctx = self.connect().await.map_err(|err| ReadError(err.0))?;

        let readed_data = match tag.command {
            Command::Coil => from_coil_to_word(ctx.read_coils(tag.address, tag.length)),
            Command::Discrete => {
                from_coil_to_word(ctx.read_discrete_inputs(tag.address, tag.length))
            }
            Command::Holding => ctx.read_holding_registers(tag.address, tag.length),
            Command::Input => ctx.read_input_registers(tag.address, tag.length),
        };

        let readed_data = readed_data
            .await
            .map_err(|err| ReadError(err.to_string()))?;

        ctx.disconnect()
            .await
            .map_err(|err| ReadError(err.to_string()))?;

        let parsed_data = parse_readed(readed_data, &tag);

        Ok(TagResponse {
            id: tag.name.clone(),
            value: parsed_data,
        })
    }

    async fn write(&self, tag: &ModbusTcpTag, value: TagValue) -> Result<(), WriteError> {
        let mut ctx = self.connect().await.map_err(|err| WriteError(err.0))?;
        let tag_to_write = tag;

        let value_to_write = match value {
            TagValue::F32(x) => {
                let scaled_value: f32 = x;
                scaled_value.to_be_bytes()
            }
            TagValue::I32(x) => {
                let scaled_value = x;
                scaled_value.to_be_bytes()
            }
        };

        dbg!(value_to_write);

        match tag_to_write.command {
            Command::Coil => {
                ctx.write_single_coil(tag_to_write.address, value_to_write.iter().sum::<u8>() != 0)
                    .await
            }
            Command::Discrete => unimplemented!("A discrete register cannot be written."),
            Command::Holding => {
                let value: Vec<u16> = value_to_write
                    .windows(2)
                    .map(|pair| {
                        let word: [u8; 2] = [pair[0].clone(), pair[1].clone()];
                        u16::from_be_bytes(word)
                    })
                    .collect();

                ctx.write_multiple_registers(tag_to_write.address, &value)
                    .await
            }
            Command::Input => unimplemented!("An input register cannot be written."),
        }
        .map_err(|err| WriteError(err.to_string()))?;

        ctx.disconnect()
            .await
            .map_err(|err| WriteError(err.to_string()))?;

        Ok(())
    }
}

use core::future::Future;
use core::pin::Pin;
fn from_coil_to_word<'a>(
    data: impl Future<Output = Result<Vec<bool>, std::io::Error>> + std::marker::Send + 'a,
) -> Pin<Box<dyn Future<Output = Result<Vec<u16>, std::io::Error>> + std::marker::Send + 'a>> {
    Box::pin(async {
        Ok(data
            .await?
            .iter()
            .map(|b| match b {
                true => 1,
                false => 0,
            })
            .collect())
    })
}

fn parse_readed(data: Vec<u16>, tag: &ModbusTcpTag) -> TagValue {
    let data: Vec<u16> = match tag.swap {
        Swap::LittleEndian => data.iter().map(|w| swap_bytes(w)).rev().collect(),
        Swap::BigEndian => data,
        Swap::LittleEndianSwap => swap_words(data.iter().map(|w| swap_bytes(w)).rev().collect()),
        Swap::BigEndianSwap => swap_words(data),
    };

    let data_as_string = match tag.data_type {
        Type::Integer => data
            .iter()
            .fold(0i32, |acc, &num| acc << 16 | num as i32)
            .to_string(),
        Type::Float => {
            let num = data.iter().fold(0u32, |acc, &num| acc << 16 | num as u32);
            format!("{:.2}", f32::from_bits(num))
        }
    };

    let readed_value: f32 = data_as_string.parse().unwrap();
    let scaled_value = readed_value * tag.multiplier;

    match is_integer(scaled_value) {
        true => TagValue::I32(scaled_value as i32),
        false => TagValue::F32(scaled_value),
    }
}

fn is_integer(value: f32) -> bool {
    value == value.round()
}

fn swap_words(words: Vec<u16>) -> Vec<u16> {
    let mut data = words.clone();
    data.swap(0, 1);
    data.to_vec()
}

fn swap_bytes(word: &u16) -> u16 {
    word.rotate_left(8)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_from_coil_to_word() {
        use super::from_coil_to_word;

        let async_response = |bool_vec: Vec<bool>| async move {
            Ok(bool_vec)
        };

        let bool_vec = vec![true, true, false, false];
        assert_eq!(vec![1, 1, 0, 0], from_coil_to_word(async_response(bool_vec)).await.unwrap());

        let bool_vec = vec![true, false, false, true];
        assert_eq!(vec![1, 0, 0, 1], from_coil_to_word(async_response(bool_vec)).await.unwrap());

        let bool_vec = vec![true, true, false, false];
        assert_ne!(vec![0, 0, 0, 0], from_coil_to_word(async_response(bool_vec)).await.unwrap());
    }

    #[test]
    fn test_parse_readed() {
        use crate::models::tag::TagReadFrequency;
        use super::{ parse_readed, ModbusTcpTag, Command, Type, Swap, TagValue };

        let mut tag = ModbusTcpTag {
            name: String::from("TEST"),
            address: 10,
            command: Command::Holding,
            data_type: Type::Integer,
            length: 1,
            multiplier: 0.1,
            read_freq: TagReadFrequency::Seconds(1),
            swap: Swap::BigEndian,
        };

        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::F32(23.2), parse_readed(u16_vec, &tag));
        
        tag.multiplier = 1.0;
        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::I32(232), parse_readed(u16_vec, &tag));

        tag.swap = Swap::BigEndianSwap;
        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::I32(15204352), parse_readed(u16_vec, &tag));

        tag.swap = Swap::BigEndianSwap;
        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::I32(15204352), parse_readed(u16_vec, &tag));

        tag.swap = Swap::LittleEndian;
        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::I32(-402653184), parse_readed(u16_vec, &tag));

        tag.swap = Swap::LittleEndianSwap;
        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(TagValue::I32(59392), parse_readed(u16_vec, &tag));


    }
}