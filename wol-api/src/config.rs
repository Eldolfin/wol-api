use anyhow::Context as _;
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use futures_util::StreamExt as _;
use inotify::{Inotify, WatchMask};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::{self, mpsc::Receiver};
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

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SshConfig {
    pub private_key_file: PathBuf,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub machines: HashMap<String, MachineCfg>,
    pub ssh: SshConfig,
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

pub fn open(
    path: &PathBuf,
    auto_reload: bool,
) -> anyhow::Result<(Arc<Mutex<Config>>, Receiver<()>)> {
    fn load_config(path: &PathBuf) -> Result<Config, anyhow::Error> {
        let config = Figment::new()
            .merge(Yaml::file(path))
            .extract()
            .with_context(|| format!("Failed to parse config file at {}", path.display()))?;
        debug!("config: {config:?}");
        Ok(config)
    }

    let shared_config = Arc::new(Mutex::new(load_config(path)?));
    let (config_changed_sender, config_changed_receiver) = sync::mpsc::channel(10);

    if auto_reload {
        let inotify = Inotify::init().expect("Failed to initialize inotify");
        inotify
            .watches()
            .add(
                path.parent().expect("Config file cannot be the root dir"),
                WatchMask::MODIFY,
            )
            .with_context(|| {
                format!("Failed to add a watcher on config file {}", path.display())
            })?;

        let path = path.to_owned();
        let shared_config = shared_config.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let mut stream = inotify.into_event_stream(&mut buffer).unwrap();
            while let Some(event_or_error) = stream.next().await {
                let event = event_or_error.expect("Err while checking for config changes");
                if event.name.unwrap() != path.file_name().unwrap() {
                    // ignore sibling file changes
                    continue;
                }
                debug!("config file changed");
                match load_config(&path) {
                    Ok(new_config) => {
                        *shared_config.lock().unwrap() = new_config;
                        if config_changed_sender.send(()).await.is_err() {
                            debug!("Config changed listener (main thread) stopped. Stopping config reloading thread");
                            break;
                        }
                        debug!("Successfully hot reloaded config");
                    }
                    Err(err) => {
                        error!("Failed to not hot-reload config: {:#}", err);
                    }
                };
            }
            info!("Stopped watching the config for changes.");
        });
    }

    Ok((shared_config, config_changed_receiver))
}
