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
    fn get_name(&self) -> &str{
        &self.name
    }
    fn get_freq(&self) -> &TagReadFrequency {
        &self.read_freq
    }
}

#[derive(Debug, Clone)]
pub struct ModbusTcpDevice(pub ModbusTcpConnection);

use crate::models::device::{ReadError, THardDevice, WriteError};
use crate::models::tag::{TagResponse, TagValue, TagReadFrequency, TValidTag};

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

        let tag_to_read = tag;

        let readed_data = match tag_to_read.command {
            Command::Coil => from_coil_to_word(
                ctx.read_coils(tag_to_read.address, tag_to_read.length)
                    .await
                    .map_err(|err| ReadError(err.to_string())),
            ),
            Command::Discrete => from_coil_to_word(
                ctx.read_discrete_inputs(tag_to_read.address, tag_to_read.length)
                    .await
                    .map_err(|err| ReadError(err.to_string())),
            ),
            Command::Holding => ctx
                .read_holding_registers(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string())),
            Command::Input => ctx
                .read_input_registers(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string())),
        }?;

        ctx.disconnect().await
            .map_err(|err| ReadError(err.to_string()))?;

        let parsed_data = parse_for_type(
            readed_data,
            tag_to_read.data_type.clone(),
            tag_to_read.swap.clone(),
        );

        let readed_value: f32 = parsed_data.parse().unwrap();
        let scaled_value = readed_value*tag_to_read.multiplier;

        let value = match is_integer(scaled_value) {
            true  => TagValue::I32(scaled_value as i32),
            false => TagValue::F32(scaled_value),
        };

        Ok(TagResponse {
            id: tag_to_read.name.clone(),
            value,
        })
    }

    async fn write(&self, _tag: &ModbusTcpTag, _value: TagValue) -> Result<(), WriteError> {
        Ok(())
    }
}

fn from_coil_to_word(data: Result<Vec<bool>, ReadError>) -> Result<Vec<u16>, ReadError> {
    Ok(data?
        .iter()
        .map(|b| match b {
            true => 1,
            false => 0,
        })
        .collect())
}

fn parse_for_type(data: Vec<u16>, data_type: Type, swap: Swap) -> String {
    let data: Vec<u16> = match swap {
        Swap::LittleEndian => data.iter().map(|w| swap_bytes(w)).rev().collect(),
        Swap::BigEndian => data,
        Swap::LittleEndianSwap => swap_words(data.iter().map(|w| swap_bytes(w)).rev().collect()),
        Swap::BigEndianSwap => swap_words(data),
    };

    match data_type {
        Type::Integer => data
            .iter()
            .fold(0u32, |acc, &num| acc << 16 | num as u32)
            .to_string(),
        Type::Float => {
            let num = data.iter().fold(0u32, |acc, &num| acc << 16 | num as u32);
            format!("{:.2}", f32::from_bits(num))
        }
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
