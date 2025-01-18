use super::{
    api::responses::{AgentComunicationError, OpenVdiError},
    application::{ApplicationInfo, GroupedApplication},
    wol,
};
use crate::{
    agent::messages::{AgentMessage, ServerMessage},
    config,
};
use anyhow::anyhow;
use anyhow::Context as _;
use futures_util::StreamExt as _;
use futures_util::{stream::SplitSink, SinkExt as _, Stream};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs as _},
    process,
    sync::{
        self,
        mpsc::{self, Receiver},
        Arc,
    },
    time::Duration,
};
use tokio::process::Command;
use utoipa::ToSchema;
use warp::filters::ws::{Message, WebSocket};

pub type Store = sync::Arc<tokio::sync::Mutex<StoreInner>>;

#[derive(Debug)]
pub struct StoreInner {
    pub machines: Vec<Machine>,
}

pub async fn recv_agent_msg<R>(websocket: &mut R) -> anyhow::Result<AgentMessage>
where
    R: Stream<Item = Result<Message, warp::Error>> + Unpin + Send,
{
    let msg = websocket
        .next()
        .await
        .context("Agent closed his websocket")?;
    decode_agent_msg(msg)
}
fn decode_agent_msg(msg: Result<Message, warp::Error>) -> anyhow::Result<AgentMessage> {
    let msg = msg.context("Failed to received agent message")?;
    let msg_str = msg
        .to_str()
        .map_err(|_empty: ()| anyhow!("Agent sent a message that was not a string"))?;
    serde_json::from_str(msg_str).context("Agent sent an incorrect formatted message")
}

impl StoreInner {
    pub fn by_name(&self, name: &str) -> Option<&Machine> {
        self.machines
            .iter()
            .find(|machine| machine.infos.name == name)
    }
    pub fn by_name_mut(&mut self, name: &str) -> Option<&mut Machine> {
        self.machines
            .iter_mut()
            .find(|machine| machine.infos.name == name)
    }

    pub fn new(config: &config::Config) -> anyhow::Result<Self> {
        let machines: anyhow::Result<Vec<Machine>> = config
            .machines
            .iter()
            .map(|(name, config)| Machine::new(config, name))
            .collect();
        Ok(Self {
            machines: machines?,
        })
    }

    pub async fn refresh_machine_state(&mut self) {
        for machine in &mut self.machines {
            machine.update_state().await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct MachineInfos {
    #[schema(example = "computer1")]
    pub name: String,
    pub state: State,
    pub tasks: Vec<Task>,
    pub vdi_opened: bool,
    pub config: config::MachineCfg,
    pub applications: Option<GroupedApplication>,
}

#[derive(Debug)]
pub struct Machine {
    pub infos: MachineInfos,
    pub addr: SocketAddr,
    applications_list: Vec<ApplicationInfo>,
    connection: Option<SplitSink<WebSocket, Message>>,
    agent_messages: Option<Receiver<AgentMessage>>,
    listen_message_task: Option<tokio::task::JoinHandle<()>>,
}

/// SAFETY: its fine :)
unsafe impl Sync for Machine {}

impl Machine {
    pub async fn update_state(&mut self) {
        self.check_agent_msg();
        self.update_status().await;
    }

    async fn update_status(&mut self) {
        let ping_res = ping_rs::send_ping_async(
            &self.addr.ip(),
            Duration::from_secs(1),
            Arc::new(&[1, 2, 3, 4]),
            None,
        )
        .await
        .is_ok();

        if !(ping_res && self.infos.state == State::On) {
            let res = ping_res
                && self
                    .ssh()
                    .args(["echo", "ok"])
                    .output()
                    .await
                    .map(|res| res.status.success())
                    .is_ok();
            self.infos.state = Self::next_state(res, ping_res, self.infos.state);
        }

        if self.infos.state == State::On {
            self.flush_tasks().await;
        }
    }

    fn next_state(res: bool, ping_res: bool, state: State) -> State {
        match (res, ping_res, state) {
            (_, true, State::PendingOff) | (false, true, State::On) => State::PendingOff,
            (true, _, _) => State::On,
            (false, true, State::Off) | (false, _, State::PendingOn) => State::PendingOn,
            (false, false, _) => State::Off,
            (false, true, State::Unknown) => State::Unknown, // we could be PendingOff or PendingOn
        }
    }
    fn ssh(&self) -> Command {
        debug!("sshing into oscar@{}", self.addr);
        let mut cmd = Command::new("ssh");
        cmd.arg("-i")
            .arg("~/.ssh/id_ed25519")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-p")
            .arg(self.addr.port().to_string())
            .arg(format!("oscar@{}", self.addr.ip()));
        cmd
    }

    pub async fn open_vdi(&mut self) -> Result<(), OpenVdiError> {
        if self.infos.vdi_opened {
            return Err(OpenVdiError::AlreadyOpened);
        }
        self.send_message(&ServerMessage::OpenVdi)
            .await
            .map_err(OpenVdiError::AgentComunicationError)?;
        self.infos.vdi_opened = true;
        Ok(())
    }
    pub async fn shutdown(&mut self, dry_run: bool) -> String {
        self.infos.state = State::PendingOff;
        let mut cmd = self.ssh();
        cmd.arg("sudo")
            // .arg("systemctl").arg("poweroff")
            .arg("poweroff");
        info!("Shutting down machine '{}'", self.infos.name);
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
            self.infos.name,
            self.infos.config.mac.to_uppercase()
        );
        self.infos.state = State::PendingOn;

        let send = wol::send(&self.infos.config.mac, dry_run);
        match send {
            Ok(()) => Ok("Sent wake on lan successfully".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn push_task(&mut self, task: Task, dry_run: bool) -> Result<String, String> {
        if task.id >= self.infos.config.tasks.len() {
            return Err(format!(
                "Task id {} is out of bound for machine {} which has {} tasks",
                task.id,
                self.infos.name,
                self.infos.tasks.len()
            ));
        }
        let name = self.infos.config.tasks[task.id].name.clone();
        debug!("Pushing task {}, to {}", name, self.infos.name);
        self.infos.tasks.push(task);
        if self.infos.state == State::Off {
            let res = self.wake(dry_run)?;
            debug!("Push task: wake on lan result: {res}");
        }
        Ok(format!("Pushed task '{name}' successfully"))
    }

    async fn flush_tasks(&mut self) {
        #[expect(
            clippy::collection_is_never_read,
            reason = "TODO: send them to front and do a popup"
        )]
        let mut errors = Vec::new(); // TODO: report them somehow
        while let Some(task) = self.infos.tasks.pop() {
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
            infos: MachineInfos {
                config: config.to_owned(),
                name: name.to_owned(),
                state: State::default(),
                tasks: vec![],
                applications: None,
                vdi_opened: false,
            },
            addr: config
                .ip
                .to_socket_addrs()
                .with_context(|| format!("Could not parse '{name}' ip"))?
                .next()
                .context("Error while resolving '{name}' ip")?,
            applications_list: vec![],
            connection: None,
            agent_messages: None,
            listen_message_task: None,
        })
    }

    pub async fn set_applications(&mut self, applications: Vec<ApplicationInfo>) {
        self.infos.applications = Some(GroupedApplication::from_list(applications.clone()).await);
        self.applications_list = applications;
    }

    pub fn set_connection(&mut self, connection: WebSocket) {
        let (ws_send, mut ws_recv) = connection.split();
        let (ch_send, ch_recv) = mpsc::channel();
        self.connection = Some(ws_send);
        self.agent_messages = Some(ch_recv);
        let name = self.infos.name.clone();
        let task = async move {
            while let Some(msg) = ws_recv.next().await {
                let msg = match decode_agent_msg(msg) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Receiving `{}`'s agent message: {:#}", &name, err);
                        continue;
                    }
                };
                debug!("Received message from `{}`'s agent: {:?}", &name, msg);
                ch_send.send(msg).expect("Backend to be alive");
            }
            debug!("Agent of `{}` disconnected", &name);
        };
        self.listen_message_task = Some(tokio::spawn(task));
    }

    pub async fn open_app(&self, application_name: &str, dry_run: bool) -> anyhow::Result<()> {
        let app_command = self
            .find_application(application_name)
            .ok_or_else(|| anyhow::anyhow!("No application found with name {application_name}"))?
            .exec
            .clone();
        if dry_run {
            return Ok(());
        }
        self.exec_desktop_cmd(&app_command)
            .await
            .with_context(|| format!("Could not open app with command {app_command}"))?;
        Ok(())
    }

    fn find_application(&self, application_name: &str) -> Option<&ApplicationInfo> {
        self.applications_list
            .iter()
            .find(|app| app.name == application_name)
    }

    async fn exec_desktop_cmd(&self, app_command: &str) -> Result<process::Output, io::Error> {
        // TODO: unhardcode display
        self.ssh()
            .arg(format!("DISPLAY=:0 {app_command} >/dev/null 2>&1 & disown"))
            .output()
            .await
    }

    async fn send_message(&mut self, msg: &ServerMessage) -> Result<(), AgentComunicationError> {
        let Some(connection) = &mut self.connection else {
            return Err(AgentComunicationError::NotConnected);
        };
        let message = Message::text(serde_json::to_string(msg).unwrap());
        connection
            .send(message)
            .await
            .with_context(|| format!("Could not send message {msg:?} to {}", self.infos.name))
            .map_err(|err| format!("{err:#}"))
            .map_err(AgentComunicationError::SendFailed)?;
        Ok(())
    }

    fn check_agent_msg(&mut self) {
        if self
            .listen_message_task
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            debug!("Stopped listening for {}'s agent messages", self.infos.name);
            self.listen_message_task = None;
            self.infos.vdi_opened = false; // agent was killed so we assume the vdi died too
        }
        if let Some(recv) = &self.agent_messages {
            if let Ok(msg) = recv.try_recv() {
                self.handle_agent_msg(msg);
            }
        }
    }

    fn handle_agent_msg(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::Hello(_) => unreachable!("it's handled in main atm"),
            AgentMessage::VdiClosed => {
                self.infos.vdi_opened = false;
            }
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    id: usize,
}
impl Task {
    async fn execute(&self, on: &Machine) -> anyhow::Result<()> {
        let res = on
            .ssh()
            .args(&on.infos.config.tasks[self.id].command)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(false, false, State::Unknown, State::Off)]
    #[case(false, false, State::Off, State::Off)]
    #[case(false, false, State::On, State::Off)]
    #[case(false, false, State::PendingOff, State::Off)]
    #[case(false, false, State::PendingOn, State::PendingOn)]
    #[case(false, true, State::Unknown, State::Unknown)]
    #[case(false, true, State::PendingOff, State::PendingOff)]
    #[case(false, true, State::PendingOn, State::PendingOn)]
    #[case(false, true, State::On, State::PendingOff)]
    #[case(false, true, State::Off, State::PendingOn)]
    #[case(true, true, State::Unknown, State::On)]
    #[case(true, true, State::On, State::On)]
    #[case(true, true, State::Off, State::On)]
    #[case(true, true, State::PendingOn, State::On)]
    #[case(true, true, State::PendingOff, State::PendingOff)]
    fn test_next_state(
        #[case] res: bool,
        #[case] ping_res: bool,
        #[case] cur_state: State,
        #[case] expected_state: State,
    ) {
        assert_eq!(
            Machine::next_state(res, ping_res, cur_state),
            expected_state
        );
    }
}
