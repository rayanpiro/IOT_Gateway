use tokio_modbus::{client::Context, prelude::*};

use crate::gen_matcher;

pub struct ModbusTcpTag(ModbusTcpConnectionParameters, ModbusTcpTagReadRequest);

#[derive(Debug)]
pub struct ModbusTcpConnectionParameters {
    pub ip_address: String,
    pub port: u32,
    pub slave: u8,
}

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

#[derive(Debug, Clone)]
pub struct ModbusTcpTagReadRequest {
    name: String,
    command: Command,
    address: u16,
    length: u16,
    swap: Swap,
    data_type: Type,
}

#[derive(Debug, Clone)]
pub struct ModbusTcpTagResponse {
    pub name: String,
    pub value: String,
}


// pub struct ModbusConnection(Context);

// #[derive(Debug)]
// pub struct ModbusError(String);
// impl std::error::Error for ModbusError {}
// impl std::fmt::Display for ModbusError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "The following error has ocurred during modbus communication: {}",
//             self.0
//         )
//     }
// }

// pub async fn modbus_connect(
//     connection_data: &ModbusTcpConnectionParameters,
// ) -> Result<ModbusConnection, ModbusError> {
//     let socket_address = format!("{}:{}", connection_data.ip_address, connection_data.port)
//         .parse()
//         .expect(
//             format!(
//                 "The ip {} or the port {} read from config file cannot be parsed as valid address.",
//                 connection_data.ip_address, connection_data.port
//             )
//             .as_str(),
//         );

//     match client::tcp::connect_slave(socket_address, Slave(connection_data.slave)).await {
//         Ok(ctx) => Ok(ModbusConnection(ctx)),
//         Err(err) => Err(ModbusError(err.to_string())),
//     }
// }

// fn parse_bool(data: Vec<bool>) -> String {
//     let data = data.first().unwrap();

//     match &data {
//         true => "True".to_string(),
//         false => "False".to_string(),
//     }
// }

// fn swap_bytes(word: &u16) -> u16 {
//     word.rotate_left(8)
// }

// fn swap_words(words: Vec<u16>) -> Vec<u16> {
//     let mut data = words.clone();
//     data.swap(0, 1);
//     data.to_vec()
// }

// fn parse_for_type(data: Vec<u16>, data_type: Type, swap: Swap) -> String {
//     let data: Vec<u16> = match swap {
//         Swap::LittleEndian => data.iter().map(|w| swap_bytes(w)).rev().collect(),
//         Swap::BigEndian => data,
//         Swap::LittleEndianSwap => swap_words(data.iter().map(|w| swap_bytes(w)).rev().collect()),
//         Swap::BigEndianSwap => swap_words(data),
//     };

//     match data_type {
//         Type::Integer => data
//             .iter()
//             .fold(0u32, |acc, &num| acc << 16 | num as u32)
//             .to_string(),
//         Type::Float => {
//             let num = data.iter().fold(0u32, |acc, &num| acc << 16 | num as u32);
//             format!("{:.2}", f32::from_bits(num))
//         }
//     }
// }

// impl ModbusConnection {
//     pub async fn modbus_read(
//         &mut self,
//         tag: ModbusTcpTagReadRequest,
//     ) -> Result<ModbusTcpTagResponse, ModbusError> {
//         match self.get_data(tag).await {
//             Ok(data) => Ok(data),
//             Err(err) => Err(ModbusError(err.to_string())),
//         }
//     }

//     async fn get_data(
//         &mut self,
//         tag: ModbusTcpTagReadRequest,
//     ) -> Result<ModbusTcpTagResponse, Box<dyn std::error::Error>> {
//         let ctx = &mut self.0;
//         let add = tag.address;
//         let length = tag.length;
//         let data_type = tag.data_type;
//         let swap = tag.swap;

//         let result = match tag.command {
//             Command::ReadCoil => parse_bool(ctx.read_coils(add, 1).await?),
//             Command::ReadDiscrete => parse_bool(ctx.read_discrete_inputs(add, 1).await?),
//             Command::ReadHolding => parse_for_type(
//                 ctx.read_holding_registers(add, length).await?,
//                 data_type,
//                 swap,
//             ),
//             Command::ReadInput => parse_for_type(
//                 ctx.read_input_registers(add, length).await?,
//                 data_type,
//                 swap,
//             ),
//         };

//         Ok(ModbusTcpTagResponse {
//             name: tag.name,
//             value: result,
//         })
//     }
// }
