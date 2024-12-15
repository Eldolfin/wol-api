pub mod api;
pub mod wol;

use crate::config;
use log::{debug, info};
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
    fn by_name_mut(&mut self, name: &str) -> Option<&mut Machine> {
        self.machines
            .iter_mut()
            .find(|machine| machine.name == name)
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
        info!("Refreshing machines states");
        for machine in &mut self.machines {
            machine.update_state().await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Machine {
    config: config::MachineCfg,
    state: State,
    #[schema(example = "computer1")]
    name: String,
}

impl Machine {
    async fn update_state(&mut self) {
        debug!("Checking state for {}", self.name);
        self.state = match self
            .ssh()
            // .arg("systemctl")
            // .arg("is-system-running")
            // .arg("--wait")
            .args(["echo", "ok"])
            .output()
            .await
            .map(|res| res.status.success())
        {
            Ok(true) => State::On,
            _ => State::Off,
        };
    }
    fn ssh(&self) -> Command {
        debug!(
            "sshing into oscar@{}:{}",
            self.config.ip, self.config.ssh_port
        );
        let mut cmd = Command::new("ssh");
        cmd.arg("-i")
            .arg("~/.ssh/id_ed25519")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-p")
            .arg(self.config.ssh_port.to_string())
            .arg(format!("oscar@{}", self.config.ip));
        cmd
    }

    async fn shutdown(&mut self, dry_run: bool) -> String {
        self.state = State::PendingOff;
        let mut cmd = self.ssh();
        cmd.arg("sudo")
            // .arg("systemctl").arg("poweroff")
            .arg("poweroff")
        ;
        info!("Shutting down machine '{}'", self.name);
        debug!(
            "Running command: {:?}{}",
            &cmd,
            if dry_run { " (dry run)" } else { "" }
        );
        if !dry_run {
            let output = cmd.output().await;
            if let Err(err) = output {
                return format!("ssh command failed: {err}");
            };
            debug!("Command output: {:?}", &output);
        }
        self.state = State::Off;
        "Shutdown machine successfully".to_owned()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[schema(example = "on")]
pub enum State {
    #[default]
    Unknown,
    On,
    Off,
    PendingOff,
}
