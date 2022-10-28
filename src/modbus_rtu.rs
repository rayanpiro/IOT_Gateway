use tokio_modbus::{client::Context, prelude::*};
use tokio_serial;

use crate::{gen_matcher, gen_readable_struct};

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
    struct ModbusRtuConnection {
        name: String,
        tty_path: String,
        baudrate: u32,
        port: u16,
        slave: u8,
    }
);

gen_readable_struct!(
    struct ModbusRtuTag {
        name: String,
        address: u16,
        length: u16,
        command: Command,
        swap: Swap,
        data_type: Type,
    }
);


pub struct ModbusRtuDevice(pub ModbusRtuConnection);

use crate::models::device::{Device, ReadError, WriteError};
use crate::models::tag::{TagResponse, TagValue, TagId, Tag};

impl Tag for ModbusRtuTag {}

#[derive(Debug, Clone)]
struct ModbusRtuError(String);

use std::net::SocketAddr;

impl ModbusRtuDevice {
    async fn connect(&self) -> Result<Context, ModbusRtuError> {

        let serial_address = tokio_serial::new(&self.0.tty_path, self.0.baudrate);
        let serial_stream = tokio_serial::SerialStream::open(&serial_address)
            .map_err(|err|ModbusRtuError(err.to_string()))?;

        match rtu::connect_slave(serial_stream, Slave(self.0.slave)).await {
            Ok(ctx) => Ok(ctx),
            Err(err) => Err(ModbusRtuError(err.to_string())),
        } 
    }
}

use async_trait::async_trait;

#[async_trait]
impl Device for ModbusRtuDevice {
    type TagType=ModbusRtuTag;
    async fn read(&self, tag: &Self::TagType) -> Result<TagResponse, ReadError> {
        let mut ctx = self.connect().await.map_err(|err| ReadError(err.0))?;

        let tag_to_read = tag;

        let readed_data = match tag_to_read.command {
            Command::Coil       => from_coil_to_word(ctx.read_coils(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string()))),
            Command::Discrete   => from_coil_to_word(ctx.read_discrete_inputs(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string()))),
            Command::Holding    => ctx.read_holding_registers(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string())),
            Command::Input      => ctx.read_input_registers(tag_to_read.address, tag_to_read.length)
                .await
                .map_err(|err| ReadError(err.to_string())),
        }?;

        let parsed_data = parse_for_type(readed_data, tag_to_read.data_type.clone(), tag_to_read.swap.clone());

        let value = match tag_to_read.data_type {
            Type::Integer => TagValue::i32(parsed_data.parse().unwrap()),
            Type::Float   => TagValue::f32(parsed_data.parse().unwrap()),
        };

        Ok(TagResponse
        {
            id: tag_to_read.name.clone(),
            value,
        })
    }

    async fn write(&self, tag: &Self::TagType, value: TagValue) -> Result<(), WriteError> {
        Ok(())
    }
}

fn from_coil_to_word(data: Result<Vec<bool>, ReadError>) -> Result<Vec<u16>, ReadError> {
    Ok(
        data?.iter()
            .map(|b| {
                match b {
                    true    => 1,
                    false   => 0,
                }
            })
            .collect()
    )
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

fn swap_words(words: Vec<u16>) -> Vec<u16> {
    let mut data = words.clone();
    data.swap(0, 1);
    data.to_vec()
}

fn swap_bytes(word: &u16) -> u16 {
    word.rotate_left(8)
}