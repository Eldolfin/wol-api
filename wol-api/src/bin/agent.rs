use std::convert;

use anyhow::Context as _;
use clap::Parser;
use log::{debug, info};
use tungstenite::connect;
use wol_relay_server::{
    agent::messages::AgentHello, machine::application::list_local_applications,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// machine name defined in the backend config eg: <oscar-tour>
    #[arg()]
    machine_name: String,
    /// backend agent-websocket ip address or domain name eg: <ws://192.168.1.1:3000>
    #[arg()]
    domain: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args {
        domain,
        machine_name,
    } = Args::parse();
    let domain = format!("{domain}/api/machine/agent");

    let applications = list_local_applications()
        .context("Could not list locally installed applications")?
        .into_iter()
        .map(convert::TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to list installed applications")?;

    let (mut socket, response) = connect(&domain)
        .with_context(|| format!("Could not connect to backend server at {domain}"))?;

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
