use tokio_modbus::{client::Context, prelude::*};

use crate::{gen_matcher, gen_readable_struct};

const SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST: u64 = 1;

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
    struct ModbusRtuOverTCPConnection {
        name: String,
        ip: std::net::IpAddr,
        port: u16,
    }
);

gen_readable_struct!(
    struct ModbusRtuTag {
        name: String,
        slave: u8,
        address: u16,
        length: u16,
        command: Command,
        swap: Swap,
        data_type: Type,
        read_freq: TagReadFrequency,
        multiplier: f32,
    }
);

impl TValidTag for ModbusRtuTag {
    fn get_name(&self) -> &str{
        &self.name
    }
    fn get_freq(&self) -> &TagReadFrequency {
        &self.read_freq
    }
}

#[derive(Debug, Clone)]
pub struct ModbusRtuOverTCPDevice(pub ModbusRtuOverTCPConnection);

use crate::models::device::{ReadError, THardDevice, WriteError};
use crate::models::tag::{TagResponse, TagValue, TagReadFrequency, TValidTag};

#[derive(Debug, Clone)]
struct ModbusRtuError(String);

impl ModbusRtuOverTCPDevice {
    async fn connect(&self, slave: Slave) -> Result<Context, ModbusRtuError> {

        let ethernet_gateway = tokio::net::TcpStream::connect((self.0.ip, self.0.port)).await
            .map_err(|err| ModbusRtuError(err.to_string()))?;

        match rtu::connect_slave(ethernet_gateway, slave).await {
            Ok(ctx) => Ok(ctx),
            Err(err) => Err(ModbusRtuError(err.to_string())),
        }
    }
}

use async_trait::async_trait;

#[async_trait]
impl THardDevice<ModbusRtuOverTCPConnection, ModbusRtuTag> for ModbusRtuOverTCPDevice {
    fn new(connection: ModbusRtuOverTCPConnection) -> Self {
        ModbusRtuOverTCPDevice(connection)
    }

    async fn read(&self, tag: &ModbusRtuTag) -> Result<TagResponse, ReadError> {
        let mut ctx = self.connect(Slave(tag.slave)).await.map_err(|err| ReadError(err.0))?;
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

        // Making this little sleep we block the handler during X time
        // this time gives the cheaper devices some more time to handle
        // the next request.
        tokio::time::sleep(std::time::Duration::new(SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST, 0)).await;

        Ok(TagResponse {
            id: tag_to_read.name.clone(),
            value,
        })
    }

    async fn write(&self, tag: &ModbusRtuTag, value: TagValue) -> Result<(), WriteError> {
        let mut ctx = self.connect(Slave(tag.slave)).await.map_err(|err| WriteError(err.0))?;
        let tag_to_write = tag;

        let value_to_write = match value {
            TagValue::F32(x) => {
                let scaled_value: f32 = x/tag_to_write.multiplier;
                scaled_value.to_be_bytes()
            },
            TagValue::I32(x) => {
                let scaled_value: f32 = (x as f32)/tag_to_write.multiplier;
                scaled_value.to_be_bytes()
            },
        };

        dbg!(value_to_write);

        match tag_to_write.command {
            Command::Coil => ctx.write_single_coil(tag_to_write.address, value_to_write.iter().sum::<u8>() != 0)
                    .await
                    .map_err(|err| WriteError(err.to_string()),
            ),
            Command::Discrete => unimplemented!("A discrete register cannot be writted."),
            Command::Holding => {
                let value: Vec<u16> = value_to_write.windows(2)
                    .map(|pair| {
                        let word: [u8; 2] = [pair[0].clone(), pair[1].clone()];
                        u16::from_be_bytes(word)
                    })
                    .collect();
                    
                ctx
                    .write_multiple_registers(tag_to_write.address, &value)
                    .await
                    .map_err(|err| WriteError(err.to_string()))
            },
            Command::Input => unimplemented!("A discrete register cannot be writted."),
        }?;

        ctx.disconnect().await
            .map_err(|err| WriteError(err.to_string()))?;

        // Making this little sleep we block the handler during X time
        // this time gives the cheaper devices some more time to handle
        // the next request.
        tokio::time::sleep(std::time::Duration::new(SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST, 0)).await;

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
