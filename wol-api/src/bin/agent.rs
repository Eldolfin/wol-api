use anyhow::{anyhow, Context as _};
use clap::Parser;
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use tokio::process::Command;
use tungstenite::{connect, Message, WebSocket};
use wol_relay_server::{
    agent::messages::{AgentHello, AgentMessage, ServerMessage},
    machine::application::{list_local_applications, Application, ApplicationInfo},
};

const MAX_RETRIES: usize = 32;
const RETRIES_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the config file
    #[arg()]
    config: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
    /// machine name defined in the backend config eg: <oscar-tour>
    machine_name: String,
    /// backend agent-websocket ip address or domain name eg: <ws://192.168.1.1:3000>
    domain: String,
    /// Shell command to run to start the vdi
    start_vdi_cmd: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args {
        config: config_path,
    } = Args::parse();
    let Config {
        machine_name,
        domain,
        start_vdi_cmd,
    } = Figment::new()
        .merge(Yaml::file(&config_path))
        .extract()
        .with_context(|| {
            debug!("Error config file content: {:#}", {
                let mut buf = String::new();
                File::open(&config_path)
                    .unwrap()
                    .read_to_string(&mut buf)
                    .unwrap();
                buf
            });
            format!("Failed to parse config file at {}", config_path.display())
        })?;
    debug!("config: {config_path:?}");

    let domain = format!("{domain}/api/machine/agent");

    info!("Listing applications...");
    let applications = list_local_applications()
        .await
        .context("Could not list locally installed applications")?;
    info!("Reading applications icons...");
    let applications: Vec<ApplicationInfo> = applications
        .into_par_iter()
        .map(Application::try_into)
        .filter_map(|res: Result<ApplicationInfo, _>| match res {
            Ok(app) => Some(app),
            Err(err) => {
                warn!("Error while listing local applications: {:#}", err);
                None
            }
        })
        .collect();

    let mut res = Err(anyhow!(
        "unreachable? because MAX_RETRIES ({MAX_RETRIES}) > 0"
    ));

    info!("Connecting to backend at {}", &domain);
    for i in 0..MAX_RETRIES {
        debug!("Try #{}/{}", i, MAX_RETRIES);
        match connect(&domain)
            .with_context(|| format!("Could not connect to backend server at {domain}"))
        {
            Ok(ok) => {
                res = Ok(ok);
                break;
            }
            Err(err) => {
                warn!("#{}: {:#}", i, err);
                res = Err(err);
                sleep(RETRIES_INTERVAL);
            }
        }
    }
    let (mut socket, response) = res?;

    info!("Connected to the server");
    debug!("Response HTTP code: {}", response.status());

    let hello = AgentMessage::Hello(AgentHello {
        machine_name,
        applications,
    });
    send_message(&mut socket, &hello)?;

    let socket = Arc::new(Mutex::new(socket));
    loop {
        let msg = socket
            .lock()
            .unwrap()
            .read()
            .context("Failed to read message from backend socket")?;
        let msg: ServerMessage = serde_json::from_str(msg.into_text().unwrap().as_str())
            .context("Expected server to send correct json messages")?;
        match msg {
            ServerMessage::OpenVdi => {
                let start_vdi_cmd = start_vdi_cmd.clone();
                let socket = socket.clone();
                tokio::spawn(async move {
                    _ = open_vdi(&start_vdi_cmd)
                        .await
                        .with_context(|| format!("Failed to open vdi (cmd = {:#})", &start_vdi_cmd))
                        .inspect_err(|err| error!("TODO: report vdi error to backend: {:#}", err));
                    send_message(&mut socket.lock().unwrap(), &AgentMessage::VdiClosed).unwrap();
                });
            }
        }
    }
    // info!("Agent is done. Exiting");
    // Ok(())
}

fn send_message<Stream>(ws: &mut WebSocket<Stream>, msg: &AgentMessage) -> anyhow::Result<()>
where
    Stream: Read + Write,
{
    ws.send(Message::Text(serde_json::to_string(msg)?.into()))
        .context("Could not send message to backend")
}

async fn open_vdi(start_vdi_cmd: &str) -> anyhow::Result<()> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(start_vdi_cmd)
        .spawn()
        .context("Failed to spawn command")?;
    let output = child.wait().await.context("vdi command failed")?;
    debug!("vdi command exited with {:?}", output);
    Ok(())
}
