use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TagResponse {
    pub id: String,
    pub value: TagValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TagValue {
    F32(f32),
    // U32(u32),
    I32(i32),
    // String(String),
}

impl ToString for TagValue {
    fn to_string(&self) -> String {
        match self {
            Self::I32(x) => x.to_string(),
            Self::F32(x) => x.to_string(),
        }
    }
}

pub trait Named: Send + Sync {
    fn name(&self) -> &str;
}

pub trait Introspection: Send + Sync {
    fn get_self(&self) -> &Self {
        &self
    }
}
