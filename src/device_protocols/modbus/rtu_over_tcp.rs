use tokio_modbus::{client::Context, prelude::*};

use crate::gen_readable_struct;
use crate::DeviceProtocols;

use super::shared;

const SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST: u64 = 1;

gen_readable_struct!(
    struct Gateway {
        name: String,
        ip: std::net::IpAddr,
        port: u16,
    }
);

gen_readable_struct!(
    struct Connection {
        name: String,
        slave: u8,
        read_freq: ReadFrequency,
    }
);

gen_readable_struct!(
    struct Tag {
        name: String,
        address: u16,
        length: u16,
        command: shared::Command,
        swap: shared::Swap,
        data_type: shared::Type,
        multiplier: f32,
    }
);

use crate::config_files::ini_parser;
use std::sync::Arc;
use tokio::sync::Mutex;
pub fn reader<F>(constructor: F, path: &str) -> Vec<DeviceProtocols>
where
    F: Fn(Arc<Mutex<Gateway>>, Connection, Tag) -> DeviceProtocols,
{
    let mut rtu_devices_under_same_gw = Vec::new();
    let gateway = Arc::new(Mutex::new(
        ini_parser::read_file::<Gateway>(&(format!("{}/connection.ini", path)))[0].to_owned(),
    ));

    for device_folder in
        std::fs::read_dir(path).unwrap_or_else(|_| panic!("The folder {} cannot be found.", &path))
    {
        let device_folder = device_folder.unwrap().path();
        if !device_folder.is_dir() {
            continue;
        }
        let path = device_folder.to_str().unwrap().to_string();

        let connection =
            ini_parser::read_file::<Connection>(&(format!("{}/connection.ini", &path)))[0]
                .to_owned();

        ini_parser::read_file::<Tag>(&(format!("{}/publishers.ini", &path)))
            .iter()
            .for_each(|tag| {
                rtu_devices_under_same_gw.push(constructor(
                    gateway.to_owned(),
                    connection.to_owned(),
                    tag.to_owned(),
                ))
            });
    }
    rtu_devices_under_same_gw
}

use crate::models::device::{ReadError, ReadFrequency, WriteError};
use crate::models::tag::{TagResponse, TagValue};

#[derive(Debug, Clone)]
struct Error(String);

async fn connect(gw: &Gateway, con: &Connection) -> Result<Context, Error> {
    let Gateway { ip, port, .. } = gw.to_owned();
    let Connection { slave, .. } = con.to_owned();

    let ethernet_gateway = tokio::net::TcpStream::connect((ip, port))
        .await
        .map_err(|err| Error(err.to_string()))?;

    match rtu::connect_slave(ethernet_gateway, Slave(slave)).await {
        Ok(ctx) => Ok(ctx),
        Err(err) => Err(Error(err.to_string())),
    }
}

pub async fn read(gw: &Gateway, con: &Connection, tag: &Tag) -> Result<TagResponse, ReadError> {
    let mut ctx = connect(gw, con).await.map_err(|err| ReadError(err.0))?;

    let raw_data = shared::read(&mut ctx, &tag.command, tag.address, tag.length).await?;
    let parsed_data = shared::parse_readed(raw_data, &tag.swap, &tag.data_type, &tag.multiplier);

    // Making this little sleep we block the handler during X time
    // this time gives the cheaper devices some more time to handle
    // the next request.
    tokio::time::sleep(std::time::Duration::new(
        SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST,
        0,
    ))
    .await;

    Ok(TagResponse {
        id: format!("{}/{}", con.name, tag.name),
        value: parsed_data,
    })
}

pub async fn write(
    gw: &Gateway,
    con: &Connection,
    tag: &Tag,
    value: TagValue,
) -> Result<(), WriteError> {
    let mut ctx = connect(gw, con).await.map_err(|err| WriteError(err.0))?;

    let value_to_write = match value {
        TagValue::F32(val) => val.to_le_bytes(),
        TagValue::I32(val) => val.to_le_bytes(),
    };

    dbg!(value_to_write);

    match tag.command {
        shared::Command::Coil => {
            ctx.write_single_coil(tag.address, value_to_write.iter().sum::<u8>() != 0)
                .await
        }

        shared::Command::Discrete => unimplemented!("A discrete register cannot be written."),

        shared::Command::Holding => {
            let value: Vec<u16> = value_to_write
                .windows(2)
                .map(|pair| {
                    let word: [u8; 2] = [pair[0], pair[1]];
                    u16::from_be_bytes(word)
                })
                .collect();
            dbg!(&value);
            ctx.write_multiple_registers(tag.address, &value).await
        }

        shared::Command::Input => unimplemented!("An input register cannot be written."),
    }
    .map_err(|err| WriteError(err.to_string()))?;

    ctx.disconnect()
        .await
        .map_err(|err| WriteError(err.to_string()))?;

    // Making this little sleep we block the handler during X time
    // this time gives the cheaper devices some more time to handle
    // the next request.
    tokio::time::sleep(std::time::Duration::new(
        SLEEP_SECONDS_CONVERTER_NEEDS_TO_HANDLE_NEW_REQUEST,
        0,
    ))
    .await;

    Ok(())
}
