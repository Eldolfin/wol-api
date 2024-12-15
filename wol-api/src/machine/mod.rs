pub mod api;
pub mod wol;

use crate::config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{process::Command, sync::Mutex};
use utoipa::ToSchema;

pub type Store = Arc<Mutex<StoreInner>>;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct StoreInner {
    machines: Vec<Machine>,
}

impl StoreInner {
    fn by_name(&self, name: &str) -> Option<Machine> {
        self.machines
            .iter()
            .find(|machine| machine.name == name)
            .cloned()
    }

    fn new(config: &config::Config) -> Self {
        let machines = config
            .machines
            .iter()
            .map(|(name, config)| Machine {
                config: config.clone(),
                state: State::default(),
                name: name.to_owned(),
            })
            .collect();
        Self { machines }
    }

    async fn refresh_machine_state(&mut self) {
        for machine in &mut self.machines {
            machine.update_state().await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Machine {
    config: config::Machine,
    state: State,
    name: String,
}

impl Machine {
    async fn update_state(&mut self) {
        self.state = match Command::new("ssh")
            .arg("-i")
            .arg("~/.ssh/id_ed25519")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg(format!("oscar@{}", self.config.ip))
            .arg("systemctl")
            .arg("is-system-running")
            .arg("--wait")
            .output()
            .await
        {
            Ok(_) => State::On,
            Err(_) => State::Off,
        };
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum State {
    #[default]
    Unknown,
    On,
    Off,
}
