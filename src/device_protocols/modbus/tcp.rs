use super::shared;
use crate::{gen_readable_struct, DeviceProtocols};
use tokio_modbus::{client::Context, prelude::*};

gen_readable_struct!(
    struct Connection {
        name: String,
        ip: std::net::IpAddr,
        port: u16,
        slave: u8,
        read_freq: device::ReadFrequency,
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
        mode: super::super::Mode,
        multiplier: f32,
    }
);

use crate::config_files::ini_parser;
pub fn reader<F>(constructor: F, path: &str) -> Vec<DeviceProtocols>
where
    F: Fn(Connection, Tag) -> DeviceProtocols,
{
    let path = path.to_string();

    let connection = ini_parser::read_file::<Connection>(&(format!("{}/connection.ini", &path)))
        .into_iter()
        .next()
        .unwrap();

    let tags = ini_parser::read_file::<Tag>(&(format!("{}/publishers.ini", &path)));
    tags.iter()
        .map(|tag| constructor(connection.to_owned(), tag.to_owned()))
        .collect()
}

use crate::models::device;
use crate::models::device::{ReadError, WriteError};
use crate::models::tag::{TagResponse, TagValue};

#[derive(Debug, Clone)]
struct Error(String);

use std::net::SocketAddr;
async fn connect(con: &Connection) -> Result<Context, Error> {
    let Connection {
        ip, port, slave, ..
    } = con.to_owned();
    let socket_address = SocketAddr::new(ip, port);

    match client::tcp::connect_slave(socket_address, Slave(slave)).await {
        Ok(ctx) => Ok(ctx),
        Err(err) => Err(Error(err.to_string())),
    }
}

pub async fn read(con: &Connection, tag: &Tag) -> Result<TagResponse, ReadError> {
    let mut ctx = connect(con).await.map_err(|err| ReadError(err.0))?;

    let raw_data = shared::read(&mut ctx, &tag.command, tag.address, tag.length).await?;
    let parsed_data = shared::parse_readed(raw_data, &tag.swap, &tag.data_type, &tag.multiplier);

    Ok(TagResponse {
        id: format!("{}/{}", con.name, tag.name),
        value: parsed_data,
    })
}

pub async fn write(con: &Connection, tag: &Tag, value: TagValue) -> Result<(), WriteError> {
    let mut ctx = connect(con).await.map_err(|err| WriteError(err.0))?;

    let value_to_write = shared::parse_write(&value, &tag.swap);
    shared::write(&mut ctx, &tag.command, tag.address, &value_to_write).await?;

    Ok(())
}
