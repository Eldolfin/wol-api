use anyhow::Context;
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug, PartialEq, Eq)]
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
    #[serde(default)]
    pub tasks: Vec<TaskCfg>,
}

fn default_ssh_port() -> u16 {
    22
}

#[derive(ToSchema, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub machines: HashMap<String, MachineCfg>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TaskCfg {
    #[schema(example = "[\"echo\", \"hello\", \"world\"]")]
    pub command: Vec<String>,
    #[schema(
        example = "https://www.pngkit.com/png/full/638-6381661_satisfactory-logo-full-color-square-number.png"
    )]
    pub icon_url: String,
    #[schema(example = "Say hello world")]
    pub name: String,
}

pub fn open(path: &PathBuf, auto_reload: bool) -> anyhow::Result<Arc<Mutex<Config>>> {
    let config: Config = Figment::new()
        .merge(Yaml::file(&path))
        .extract()
        .with_context(|| format!("Failed to parse config file at {}", path.display()))?;

    Ok(Arc::new(Mutex::new(config)))
}
