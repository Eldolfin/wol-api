use std::{io::Cursor, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use futures_util::{SinkExt as _, StreamExt as _};
use log::{debug, error};
use russh::{
    client::{self},
    keys::{key::PrivateKeyWithHashAlg, load_secret_key, ssh_key},
    ChannelMsg,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use warp::{
    filters::ws::{self, Message, WebSocket},
    reject::Rejection,
    reply::Reply,
    Filter,
};

use crate::{config::Config, machine::service::Store};

#[derive(OpenApi)]
#[openapi(
    paths(connect),
    components(schemas(SshServerMessage, SshClientMessage))
)]
pub struct Api;

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SshServerMessageType {
    TerminalData(String),
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct SshServerMessage {
    pub message: SshServerMessageType,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SshClientMessageType {
    /// The client changed the size of the terminal
    #[schema(example = "json!((80, 32))")]
    ChangeSize((u32, u32)),
    /// The client typed something in the terminal
    #[schema(example = "echo hello")]
    Input(String),
}

/// Json message sent by the client's terminal
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct SshClientMessage {
    pub message: SshClientMessageType,
}

#[utoipa::path(
    get,
    path = "/{name}/connect",
    responses(
        (status = 101, description = "Switch to websocket and transfer terminal data")
    ),
    params(
        ("name" = String, Path, description = "Name of the machine to wake")
    ),
)]

// TODO: refactor the logic in a service
async fn connect(
    ssh_private_key_path: PathBuf,
    machine_name: &str,
    store: Store,
    websocket: WebSocket,
) {
    // TODO: unhardcode size, it should be transfered by the client in the ws
    const W: u32 = 80;
    const H: u32 = 60;

    // TODO: move as much initialisation as possible in a constructor?
    // TODO: keep in mind that the config is hot reloading
    const USER: &str = "oscar";
    let (mut tx, mut rx) = websocket.split();
    let key_pair = load_secret_key(ssh_private_key_path, None).unwrap();
    let config = Arc::new(client::Config::default());
    let sh = Client {};
    let machine = store
        .lock()
        .await
        .by_name(machine_name)
        .expect("TODO: send back error");
    let addrs = (machine.ip, machine.config.ssh_port);
    let mut session = client::connect(config, addrs, sh)
        .await
        .expect("TODO: handle fail to connect to ssh");
    let auth_res = session
        .authenticate_publickey(
            USER,
            PrivateKeyWithHashAlg::new(Arc::new(key_pair), None).unwrap(),
        )
        .await
        .unwrap();

    assert!(auth_res, "Authentication (with publickey) failed");

    let mut channel = session.channel_open_session().await.unwrap();

    channel
        .request_pty(false, "xterm", W, H, 0, 0, &[])
        .await
        .unwrap();
    channel.exec(true, "$0").await.unwrap();

    loop {
        tokio::select! {
            client_data = rx.next() => {
                match client_data {
                    Some(Ok(data)) => {
                        if data.is_binary() {
                            channel.data(data.as_bytes()).await.unwrap();
                        }
                        else {
                            let client_message: SshClientMessage = match serde_json::from_str(data.to_str().unwrap()) {

                                Ok(msg) => msg,
                                Err(parse_error) => {
                                    error!("could not parse client message: {parse_error}"); break;},
                            };
                            match client_message.message {
                                SshClientMessageType::Input(input) => channel.data(Cursor::new(input)).await.unwrap(),
                                SshClientMessageType::ChangeSize((cols, rows)) => {
                                    channel.window_change( cols, rows, 0, 0).await.unwrap();
                                }
                            }
                        }
                    },
                    Some(Err(_)) => unreachable!("Idk how this branch can be reached..."),
                    _ => {
                        debug!("ssh session: client disconnected");
                        break;
                    },
                };
            },
            Some(msg) = channel.wait() => {
                match msg {
                    // Write data to the terminal
                    ChannelMsg::Data { data } => {
                        tx.send(Message::binary(data.to_vec())).await.unwrap();
                    },
                    // The command has returned an exit code
                    ChannelMsg::ExitStatus { exit_status } => {
                        debug!("ssh session ended with exit code {exit_status}");
                        channel.eof().await.unwrap();
                        break;
                    }
                    ChannelMsg::Success => (),
                    other => {debug!("unhandled ssh channelmsg: {other:?}");}
                }
           }
        }
    }
}

struct Client;

#[async_trait]
#[expect(
    clippy::missing_trait_methods,
    reason = "we don't need to hook to every event"
)]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[expect(clippy::let_and_return, reason = "might add more endpoints here later")]
pub fn handlers(
    config: &Config,
    store: Store,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let connect = {
        let store = store.clone();
        let ssh_private_key_path = config.ssh.private_key_file.clone();
        warp::path!(String / "connect")
            .and(warp::ws())
            .map(move |name: String, ws: ws::Ws| {
                // And then our closure will be called when it completes...
                let store = store.clone();
                let ssh_private_key_path = ssh_private_key_path.clone();
                ws.on_upgrade(move |websocket| {
                    let store = store.clone();
                    let ssh_private_key_path = ssh_private_key_path.clone();
                    async move {
                        connect(ssh_private_key_path, &name, store, websocket).await;
                    }
                })
            })
    };

    connect
}
