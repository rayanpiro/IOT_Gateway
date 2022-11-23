use crate::device_protocols::modbus_rtu::{
    ModbusRtuOverTCPConnection, ModbusRtuOverTCPDevice, ModbusRtuTag,
};

use crate::device_protocols::modbus_tcp::{ModbusTcpConnection, ModbusTcpDevice, ModbusTcpTag};
use crate::models::{
    device::THardDevice,
    tag::{TTag, TValidTag, TagId},
};

use std::{collections::HashMap, fmt::Debug, fs, marker::PhantomData};

use std::sync::Arc;
use tokio::sync::Mutex;

const INI_PROTOCOL_FOLDERS: [&str; 2] = ["modbus_tcp/", "modbus_rtu/"];

pub fn get_tags_from_ini_files() -> Vec<Arc<dyn TTag>> {
    let mut tags: Vec<Arc<dyn TTag>> = Vec::new();

    for folder in INI_PROTOCOL_FOLDERS {
        let protocol = &folder[..folder.len() - 1];

        for device in fs::read_dir(folder).unwrap() {
            let path = device.unwrap().path().to_str().unwrap().to_string();

            match protocol {
                "modbus_tcp" => tags.append(&mut read_tags::<
                    ModbusTcpDevice,
                    ModbusTcpConnection,
                    ModbusTcpTag,
                >(&path)),
                "modbus_rtu" => tags.append(&mut read_tags::<
                    ModbusRtuOverTCPDevice,
                    ModbusRtuOverTCPConnection,
                    ModbusRtuTag,
                >(&path)),
                _ => unimplemented!(),
            }
        }
    }

    tags
}

fn read_tags<T, C, S>(path: &str) -> Vec<Arc<dyn TTag>>
where
    T: THardDevice<C, S> + Clone + Send + Sync + 'static,
    C: TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <C as TryFrom<HashMap<String, String>>>::Error: Debug,
    S: TValidTag + TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <S as TryFrom<HashMap<String, String>>>::Error: Debug,
{
    let path = path.to_string();

    use super::ini_parser;
    let connection = ini_parser::read_file::<C>(&(format!("{}/connection.ini", &path)))
        .into_iter()
        .nth(0)
        .unwrap();

    let mutex_device = Arc::new(Mutex::new(T::new(connection)));

    ini_parser::read_file::<S>(&(format!("{}/publishers.ini", &path)))
        .into_iter()
        .map(|t| {
            let tag: Arc<dyn TTag> = Arc::new(TagId {
                handler: Arc::clone(&mutex_device),
                tag: Arc::new(t),
                _phantom: PhantomData,
            });
            tag
        })
        .collect()
}

