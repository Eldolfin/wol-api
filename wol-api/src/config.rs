use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct MachineCfg {
    #[schema(example = "192.168.1.4")]
    pub ip: String,
    #[schema(example = "f4:93:9f:eb:56:a8")]
    pub mac: String,
    #[schema(example = 22)]
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
}

fn default_ssh_port() -> u16 {
    22
}

#[derive(ToSchema, Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub machines: HashMap<String, MachineCfg>,
}
