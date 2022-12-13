use crate::{
    gen_matcher,
    models::{
        device::{ReadError, ReadFrequency, WriteError},
        tag::{TagResponse, TagValue},
    },
};

pub mod modbus;

macro_rules! get_config_folders {
    ($pub_or:vis enum $e_name:ident { $( $variant:ident( $($types:ty),+ ) : config_folder: $config_folder:literal, reader: $reader:expr ),*, }) => {
        #[derive(Debug, Clone)]
        $pub_or enum $e_name {
            $( $variant($($types),+) ),*
        }

        impl $e_name {
            pub fn config_folders() -> Vec<&'static str> {
                vec![$( $config_folder ),*,]
            }

            pub fn from_ini_files() -> Vec<$e_name> {
                let mut tags = Vec::new();
                for folder in $e_name::config_folders() {
                    for device in std::fs::read_dir(folder).unwrap_or_else(|_| panic!("The folder {} cannot be found.", folder)) {
                        let protocol = &folder[..folder.len()];

                        let device_folder = device.unwrap().path();
                        if !device_folder.is_dir() {
                            continue;
                        }
                        let path = device_folder.to_str().unwrap().to_string();

                        match protocol {
                            $( $config_folder => {
                                let mut devices = $reader(DeviceProtocols::$variant, &path);
                                tags.append(&mut devices);
                            }),*,
                            _ => unimplemented!(),
                        }
                    }
                }
                tags
            }
        }
    };
}

gen_matcher!(
    enum Mode {
        Read,
        Write,
    }
);

use std::sync::Arc;
use tokio::sync::Mutex;
get_config_folders!(
    pub enum DeviceProtocols {
        ModbusTCP(modbus::tcp::Connection, modbus::tcp::Tag) : config_folder: "modbus_tcp", reader: modbus::tcp::reader,
        ModbusRTUOverTCP(Arc<Mutex<modbus::rtu_over_tcp::Gateway>>, modbus::rtu_over_tcp::Connection, modbus::rtu_over_tcp::Tag)
            : config_folder: "modbus_rtu_over_tcp", reader: modbus::rtu_over_tcp::reader,
    }
);

impl DeviceProtocols {
    pub async fn read(&self) -> Result<TagResponse, ReadError> {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(gw, c, t) => {
                modbus::rtu_over_tcp::read(&*gw.lock().await, c, t).await
            }
            DeviceProtocols::ModbusTCP(c, t) => modbus::tcp::read(c, t).await,
        }
    }

    pub async fn write(&self, value: TagValue) -> Result<(), WriteError> {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(gw, c, t) => {
                modbus::rtu_over_tcp::write(&*gw.lock().await, c, t, value).await
            }
            DeviceProtocols::ModbusTCP(c, t) => modbus::tcp::write(c, t, value).await,
        }
    }

    pub fn tag_name(&self) -> String {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(_, _, t) => t.name.to_owned(),
            DeviceProtocols::ModbusTCP(_, t) => t.name.to_owned(),
        }
    }

    pub fn device_name(&self) -> String {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(_, c, _) => c.name.to_owned(),
            DeviceProtocols::ModbusTCP(c, _) => c.name.to_owned(),
        }
    }

    pub fn mode(&self) -> Mode {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(_, _, t) => t.mode.to_owned(),
            DeviceProtocols::ModbusTCP(_, t) => t.mode.to_owned(),
        }
    }

    pub fn _name(&self) -> String {
        // &format!("{}/{}", self.device_name(), self.tag_name())
        self.tag_name()
    }

    pub fn freq(&self) -> ReadFrequency {
        match self {
            DeviceProtocols::ModbusRTUOverTCP(_, c, _) => c.read_freq.to_owned(),
            DeviceProtocols::ModbusTCP(c, _) => c.read_freq.to_owned(),
        }
    }
}
