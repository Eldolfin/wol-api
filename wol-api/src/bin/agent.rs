use anyhow::anyhow;
use anyhow::Context as _;
use clap::Parser;
use figment::providers::Yaml;
use figment::{providers::Format as _, Figment};
use log::{debug, info, warn};
use rayon::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::Read as _;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use tungstenite::connect;
use wol_relay_server::{
    agent::messages::AgentHello,
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

    let hello = AgentHello {
        machine_name,
        applications,
    };
    socket
        .send(tungstenite::Message::text(serde_json::to_string(&hello)?))
        .context("Failed to send message")?;

    // loop {
    //     let msg = socket
    //         .read()
    //         .context("Failed to read message from backend socket")?;
    //     println!("Received: {msg}");
    // }

    info!("Agent is done. Exiting");

    Ok(())
}
