use crate::modbus_tcp::modbus::{
    ModbusTcpConnectionParameters, ModbusTcpTag, ModbusTcpTagReadRequest,
};

pub enum TagProtocol {
    ModbusTcpTag,
}

pub struct TagName(String);

pub struct TagResponse {
    name: TagName,
    value: TagValue,
}

pub enum TagValue {
    f32,
    u32,
    i32,
    String,
}

pub enum TagReadFrequency {
    Miliseconds(u32),
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u32),
}

pub struct TagInfo {
    protocol: TagProtocol,
    name: TagName,
}

pub struct TagReadSync {
    tag: TagInfo,
    read_frequency: u32,
}

pub struct TagReadRequest {
    tag: TagInfo,
}

pub struct TagEvent {
    tag: TagInfo,
}

pub struct TagWrite {
    tag: TagInfo,
    value: TagValue,
}
