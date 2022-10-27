mod modbus_tcp;
mod models;
mod ini_parser;
use models::device::Device;
use tokio;

const INI_PROTOCOL_FOLDERS: [&str; 1] = ["modbus_tcp/"]; //, "modbus_rtu/"];ç

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

// struct Devices<T: models::device::Device> {
//     handler: T,
//     tags: Vec<()>,
// }

use std::{fs, collections::HashMap};
use modbus_tcp::{ModbusTcpConnection, ModbusTcpTag, ModbusTcpDevice};
use models::tag::TagId;

use std::sync::{Arc, Mutex};
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    for folder in INI_PROTOCOL_FOLDERS {
        let protocol = &folder[..folder.len()-1];
        dbg!(protocol);
        for device in fs::read_dir(folder).unwrap() {
            let path = device.unwrap().path().to_str().unwrap().to_string();
            let data = ini_parser::read_file::<ModbusTcpConnection>(&(path.clone()+"/connection.ini"));
            let tags = ini_parser::read_file::<ModbusTcpTag>(&(path+"/publishers.ini"));
            
            let tag: TagId = TagId { 
                id: tags[0].name.clone(),
                handler: Arc::new(Mutex::new(Box::new(ModbusTcpDevice(data[0].clone(), tags))))
            };

            dbg!(
                tag.handler.lock()
                    .unwrap()
                    .read(tag.clone())
                    .await
            );
            
        }
    }
    Ok(())
}
