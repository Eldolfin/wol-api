use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub struct Machine {
    #[schema(example = "192.168.1.4")]
    pub ip: String,
    #[schema(example = "f4:93:9f:eb:56:a8")]
    pub mac: String,
}

#[derive(ToSchema, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub machines: HashMap<String, Machine>,
}
