mod modbus_tcp;
mod models;
use modbus_tcp::modbus::{Command, Swap, Type};
mod ini_parser;
use models::tag::TagReadRequest;
use tokio;

const INI_PROTOCOL_FOLDERS: [&str; 1] = ["modbus_tcp/"]; //, "modbus_rtu/"];ç


struct AllowedProtocols {
    path: &'static str,
    
}

/*
Device {
    name
    ParametrosConexion
    Vec<Tags>
    Vec<Events>
    connect()
    ping()
    read()
    write()
    disconnect()
}
*/
// struct ModbusTcp;

// impl models::device::Device for ModbusTcp {
//     fn read(tag: TagInfo) -> Result<TagResponse, ReadError> {
//         Ok(TagResponse {
//             name: "A".to_string(),
//             value: "A".to_string(),
//         })
//     }
//     fn write(tag: TagWrite, value: TagValue) -> Result<(), WriteError> {
//         Ok(())
//     }
// }

struct Devices<T: models::device::Device> {
    handler: T,
    tags: Vec<()>,
}

gen_readable_struct!(
    struct ModbusTcpConnection {
        name: String,
        ip: std::net::IpAddr,
        port: u32,
        slave: u32,
    }
);

gen_readable_struct!(
    struct ModbusRtuConnection {
        name: String,
        baudrate: u32,
        parity: u32,
        odd: bool,
        slave: u32,
    }
);

gen_readable_struct!(
    struct ModbusTcpTag {
        name: String,
        address: u32,
        length: u8,
        command: Command,
        swap: Swap,
        data_type: Type,
    }
);

use std::{fs, collections::HashMap};
fn main() {
    // let mut devices: Devices<ModbusTcp>;
    for folder in INI_PROTOCOL_FOLDERS {
        let protocol = &folder[..folder.len()-1];
        dbg!(protocol);
        for device in fs::read_dir(folder).unwrap() {
            let path = device.unwrap().path().to_str().unwrap().to_string();
            let data = ini_parser::read_file::<ModbusTcpConnection>(&(path.clone()+"/connection.ini"));
            let tags = ini_parser::read_file::<ModbusTcpTag>(&(path+"/publishers.ini"));
            dbg!(data);
            dbg!(tags);
        }
    }    
}

// #[tokio::main(flavor = "current_thread")]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {

//     let program_name = std::env::args().nth(0).unwrap();
//     let ini_file = std::env::args()
//         .nth(1)
//         .expect(format!("This should be called: {} config_file.ini", program_name).as_str());

//     let (connection_parameters, tags) = modbus_tcp::ini_parser::IniFile(ini_file).get_ini_data();
//     let mut connection = modbus_tcp::modbus::modbus_connect(&connection_parameters).await?;

//     for tag in tags {
//         let response = connection.modbus_read(tag).await?;
//         println!("Slave {} - {}: {}", &connection_parameters.slave, response.name, response.value)
//     }
//     Ok(())
// }

