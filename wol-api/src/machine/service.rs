use super::wol;
use crate::config;
use anyhow::Context;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    net::IpAddr,
    sync::{self, Arc},
    time::Duration,
};
use tokio::{process::Command, sync::Mutex};
use utoipa::ToSchema;

pub type Store = sync::Arc<Mutex<StoreInner>>;

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

    pub fn new(config: &config::Config) -> anyhow::Result<Self> {
        let machines: anyhow::Result<Vec<Machine>> = config
            .machines
            .iter()
            .map(|(name, config)| Machine::new(config, name))
            .collect();
        Ok(Self { machines: machines? })
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
    pub tasks: Vec<Task>,
    #[serde(skip_serializing)]
    ip: IpAddr,
}

impl Machine {
    pub async fn update_state(&mut self)  {
        debug!("Checking state for {}", self.name);

        let ping_res = ping_rs::send_ping_async(
            &self.ip,
            Duration::from_secs(1),
            Arc::new(&[1,2,3,4]),
            None,
        )
        .await.is_ok();

        if !(ping_res && self.state == State::On) {
            let res = ping_res && self
                .ssh()
                .args(["echo", "ok"])
                .output()
                .await
                .map(|res| res.status.success()).is_ok();
            self.state = match (res,ping_res, self.state) {
                (true, _, _) => State::On,
                (false, false, _) => State::Off,
                (false, true, State::On | State::Unknown | State::PendingOff) => State::PendingOff,
                (false, true, State::Off | State::PendingOn) => State::PendingOn,
            };
        }

        if self.state == State::On {
            self.flush_tasks().await;
        }
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
        "Send shutdown command to machine successfully".to_owned()
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

    pub fn push_task(&mut self, task: Task) {
        debug!("Pushing task {:?}, to {:?}", task, self);
        self.tasks.push(task);
    }

    async fn flush_tasks(&mut self) {
        #[allow(clippy::collection_is_never_read)]
        let mut errors = Vec::new(); // TODO: report them somehow
        while let Some(task) = self.tasks.pop() {
            let res = task
                .execute(self)
                .await
                .with_context(|| format!("Failed to execute task {task:?}"));
            if let Err(err) = res {
                errors.push(err);
            }
        }
    }

    fn new(config: &config::MachineCfg, name: &str) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.to_owned(),
            name: name.to_owned(),
            ip: config
                .ip
                .parse()
                .with_context(|| format!("Could not parse '{name}' ip"))?,
            state: State::default(),
            tasks: Vec::new(),
        })
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    id: usize,
}
impl Task {
    async fn execute(&self, on: &Machine) -> anyhow::Result<()> {
        let res = on
            .ssh()
            .args(&on.config.tasks[self.id].command)
            .output()
            .await?;
        if !res.status.success() {
            anyhow::bail!(
                "stderr: {}\nstdout: {}\nreturn code: {}",
                String::from_utf8(res.stderr).unwrap(),
                String::from_utf8(res.stdout).unwrap(),
                res.status
            );
        }
        Ok(())
    }
}
