use super::wol;
use crate::config;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{process::Command, sync::Mutex};
use utoipa::ToSchema;

pub type Store = std::sync::Arc<Mutex<StoreInner>>;

pub const TIME_BEFORE_ASSUMING_WOL_FAILED: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct StoreInner {
    machines: Vec<Machine>,
}

impl StoreInner {
    pub fn by_name(&self, name: &str) -> Option<Machine> {
        self.machines
            .iter()
            .find(|machine| machine.name == name)
            .cloned()
    }
    pub fn by_name_mut(&mut self, name: &str) -> Option<&mut Machine> {
        self.machines
            .iter_mut()
            .find(|machine| machine.name == name)
    }

    pub fn new(config: &config::Config) -> Self {
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

    pub async fn refresh_machine_state(&mut self) {
        info!("Refreshing machines states");
        for machine in &mut self.machines {
            machine.update_state().await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Machine {
    pub config: config::MachineCfg,
    pub state: State,
    #[schema(example = "computer1")]
    pub name: String,
}

impl Machine {
    pub async fn update_state(&mut self) {
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

    pub async fn shutdown(&mut self, dry_run: bool) -> String {
        self.state = State::PendingOff;
        let mut cmd = self.ssh();
        cmd.arg("sudo")
            // .arg("systemctl").arg("poweroff")
            .arg("poweroff");
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

    pub fn wake(&mut self, dry_run: bool) -> Result<String, String> {
        info!(
            "Sending wake on lan to {} (mac = {})",
            self.name,
            self.config.mac.to_uppercase()
        );
        self.state = State::PendingOn;

        let send = wol::send(&self.config.mac, dry_run);
        match send {
            Ok(()) => Ok("Sent wake on lan successfully".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Default,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Hash,
)]
#[serde(rename_all = "snake_case")]
#[schema(example = "on")]
pub enum State {
    #[default]
    Unknown,
    On,
    Off,
    PendingOn,
    PendingOff,
}
