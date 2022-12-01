use super::ini_parser;
use crate::cloud_protocols::mqtt::MqttIniConfig;
use crate::device_protocols::modbus;
use crate::models::{
    device::THardDevice,
    tag::{Named, TTag, TagId},
};
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug, fs, marker::PhantomData};
use tokio::sync::Mutex;

const INI_PROTOCOL_FOLDERS: [&str; 2] = ["modbus_tcp/", "modbus_rtu_over_tcp/"];

pub fn get_mqtt_config() -> MqttIniConfig {
    ini_parser::read_file::<MqttIniConfig>("mqtt.ini")
        .into_iter()
        .next()
        .expect("Invalid mqtt.ini file")
}

pub fn get_tags_from_ini_files() -> Vec<Arc<dyn TTag>> {
    let mut tags: Vec<Arc<dyn TTag>> = Vec::new();

    for folder in INI_PROTOCOL_FOLDERS {
        let protocol = &folder[..folder.len() - 1];

        for device in fs::read_dir(folder).unwrap() {
            let path = device.unwrap().path().to_str().unwrap().to_string();

            match protocol {
                "modbus_tcp" => tags.append(&mut read_tags::<
                    modbus::tcp::Device,
                    modbus::tcp::Connection,
                    modbus::tcp::Tag,
                >(&path)),
                "modbus_rtu_over_tcp" => tags.append(&mut read_tags::<
                    modbus::rtu_over_tcp::Device,
                    modbus::rtu_over_tcp::Connection,
                    modbus::rtu_over_tcp::Tag,
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
    S: Named + TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <S as TryFrom<HashMap<String, String>>>::Error: Debug,
{
    let path = path.to_string();

    let connection = ini_parser::read_file::<C>(&(format!("{}/connection.ini", &path)))
        .into_iter()
        .next()
        .unwrap();

    let hard_device = T::new(connection);
    let device_name = hard_device.get_device_name();
    let mutex_device = Arc::new(Mutex::new(hard_device));

    ini_parser::read_file::<S>(&(format!("{}/publishers.ini", &path)))
        .into_iter()
        .map(|t| {
            let tag: Arc<dyn TTag> = Arc::new(TagId {
                handler: Arc::clone(&mutex_device),
                device_name: device_name.clone(),
                tag: Arc::new(t),
                _phantom: PhantomData,
            });
            tag
        })
        .collect()
}
