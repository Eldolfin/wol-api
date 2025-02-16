use anyhow::{anyhow, Context as _, Error};
use clap::Parser;
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use futures_util::{stream::SplitSink, SinkExt as _, StreamExt as _};
use inotify::{Inotify, WatchMask};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Read as _,
    path::PathBuf,
    sync::Arc,
    thread::sleep,
    time::Duration,
};
use tokio::process::Command;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    client_async_tls_with_config,
    tungstenite::{client::IntoClientRequest, handshake::client::Response, protocol::Message},
    Connector, MaybeTlsStream, WebSocketStream,
};
use wol_relay_server::{
    agent::messages::{AgentHello, AgentMessage, ServerMessage},
    machine::application::{list_local_applications, Application, ApplicationInfo},
};

const MAX_RETRIES: usize = 32;
const RETRIES_INTERVAL: Duration = Duration::from_secs(1);

type Socket = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;

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
            .await
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
    let (socket, response) = res?;
    let (sock_send, sock_recv) = socket.split();
    let sock_send = Arc::new(Mutex::new(sock_send));
    let sock_recv = Arc::new(Mutex::new(sock_recv));

    info!("Connected to the server");
    debug!("Response HTTP code: {}", response.status());

    let hello = AgentMessage::Hello(AgentHello {
        machine_name,
        applications,
    });
    send_message(&sock_send, &hello).await?;

    loop {
        let mut lock = sock_recv.lock().await;
        let msg = lock
            .next()
            .await
            .context("Failed to read message from backend socket")?
            .context("Failed to read message from backend socket")?;
        drop(lock);
        let msg: ServerMessage = serde_json::from_str(msg.into_text().unwrap().as_str())
            .context("Expected server to send correct json messages")?;
        match msg {
            // fun fact: we don't even check that the vdi is not already opened
            // we trust the backend to know this for us otherwise we would open
            // multiple sanzu server
            ServerMessage::OpenVdi => {
                let start_vdi_cmd = start_vdi_cmd.clone();
                let socket = sock_send.clone();
                tokio::spawn(async move {
                    if let Err(err) = open_vdi(&socket, &start_vdi_cmd)
                        .await
                        .with_context(|| format!("Failed to open vdi (cmd = {:#})", &start_vdi_cmd))
                    {
                        error!("TODO: report vdi error to backend: {:#}", err);
                    }
                    let res = send_message(&socket, &AgentMessage::VdiClosed).await;
                    if let Err(err) = res {
                        error!("Couldn't send vdi closed to backend: {:#}", err);
                    }
                });
            }
        }
    }
    // info!("Agent is done. Exiting");
    // Ok(())
}

async fn send_message(ws: &Socket, msg: &AgentMessage) -> anyhow::Result<()> {
    debug!("Sending msg to backend");
    ws.lock()
        .await
        .send(Message::Text(serde_json::to_string(msg)?))
        .await
        .context("Could not send message to backend")
}

async fn open_vdi(socket: &Socket, start_vdi_cmd: &str) -> anyhow::Result<()> {
    let certificate_hash_path: PathBuf = "/tmp/sanzu/webtransport-cert-hash.txt".into();
    fs::remove_file(&certificate_hash_path).with_context(|| {
        format!(
            "Could not delete certificate hash file at {}",
            certificate_hash_path.display(),
        )
    })?;
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(start_vdi_cmd)
        .spawn()
        .context("Failed to spawn command")?;
    let inotify = Inotify::init().expect("Failed to initialize inotify");
    inotify
        .watches()
        .add(
            certificate_hash_path.parent().unwrap(),
            WatchMask::CLOSE_WRITE,
        )
        .with_context(|| {
            format!(
                "Failed to add a watcher on certificate file {}",
                certificate_hash_path.display()
            )
        })?;
    let mut buffer = [0; 1024];
    let mut stream = inotify.into_event_stream(&mut buffer).unwrap();
    loop {
        let _ev = stream.next();
        if let Some(hash) = fs::read_to_string(&certificate_hash_path)
            .ok()
            .and_then(|hash_str| serde_json::from_str(&hash_str).ok())
        {
            send_message(socket, &AgentMessage::VdiCertificateHash(hash)).await?;
            break;
        }
    }

    let output = child.wait().await.context("vdi command failed")?;
    debug!("vdi command exited with {:?}", output);
    Ok(())
}

async fn connect<R>(
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;
    let domain = &request.uri().host().context("domain to have a hostname")?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .context("Unexpected url scheme")?;

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    let ccc = Some(Connector::Rustls(std::sync::Arc::new(config)));

    let addr = format!("{domain}:{port}");
    let socket = TcpStream::connect(addr).await?;

    client_async_tls_with_config(request, socket, None, ccc)
        .await
        .context("Could not upgrade to websocket")
}

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
