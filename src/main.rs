mod ini_parser;
mod modbus_rtu;
mod modbus_tcp;
mod models;
use tokio;

const INI_PROTOCOL_FOLDERS: [&str; 2] = ["modbus_tcp/", "modbus_rtu/"];

use modbus_rtu::{ModbusRtuConnection, ModbusRtuDevice, ModbusRtuTag};
use modbus_tcp::{ModbusTcpConnection, ModbusTcpDevice, ModbusTcpTag};
use models::{
    device::THardDevice,
    tag::{TTag, TagId},
};
use std::{collections::HashMap, fmt::Debug, fs, marker::PhantomData};

fn read_tags<T, C, S>(path: &str) -> Vec<Box<dyn TTag>>
where
    T: THardDevice<C, S> + Clone + Send + Sync + 'static,
    C: TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <C as TryFrom<HashMap<String, String>>>::Error: Debug,
    S: TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <S as TryFrom<HashMap<String, String>>>::Error: Debug,
{
    let path = path.to_string();
    let connection = ini_parser::read_file::<C>(&(format!("{}/connection.ini", &path)))
        .into_iter()
        .nth(0)
        .unwrap();

    ini_parser::read_file::<S>(&(format!("{}/publishers.ini", &path)))
        .into_iter()
        .map(|t| {
            let device = T::new(connection.clone());
            let tag: Box<dyn TTag> = Box::new(TagId {
                handler: device.clone(),
                tag: t.clone(),
                _phantom: PhantomData,
            });
            tag
        })
        .collect()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tags: Vec<Box<dyn TTag>> = Vec::new();

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
                    ModbusRtuDevice,
                    ModbusRtuConnection,
                    ModbusRtuTag,
                >(&path)),
                _ => unimplemented!(),
            }
        }
    }

    loop {
        use futures::FutureExt;
        let h1 = tags
            .iter()
            .map(|t| t.read().then(|f| async { dbg!(f.unwrap()) }));

        use futures::future::join_all;
        let _ = join_all(h1).await;
        let _ = tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
    }
}
