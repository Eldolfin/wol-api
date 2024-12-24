use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use futures_util::{SinkExt as _, StreamExt as _};
use log::debug;
use russh::{
    client::{self},
    keys::{key::PrivateKeyWithHashAlg, load_secret_key, ssh_key},
    ChannelMsg,
};
use utoipa::OpenApi;
use warp::{
    filters::ws::{self, Message, WebSocket},
    reject::Rejection,
    reply::Reply,
    Filter,
};

use crate::{config::Config, machine::service::Store};

#[derive(OpenApi)]
#[openapi(paths(connect))]
pub struct Api;

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
async fn connect(
    ssh_private_key_path: PathBuf,
    machine_name: &str,
    store: Store,
    websocket: WebSocket,
) {
    // TODO: move as much initialisation as possible in a constructor?
    // TODO: keep in mind that the config is hot reloading
    const USER: &str = "oscar";
    let (mut tx, mut rx) = websocket.split();
    let key_pair = load_secret_key(ssh_private_key_path, None).unwrap();
    let config = client::Config {
        // inactivity_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };
    let config = Arc::new(config);
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

    if !auth_res {
        panic!("Authentication (with publickey) failed");
    }

    let mut channel = session.channel_open_session().await.unwrap();

    // TODO: unhardcode size, it should be transfered by the client in the ws
    const W: u32 = 80;
    const H: u32 = 60;

    channel
        .request_pty(false, "xterm", W, H, 0, 0, &[])
        .await
        .unwrap();
    channel.exec(true, "$0").await.unwrap();

    loop {
        tokio::select! {
            r = rx.next()=> {
                match r {
                    Some(Ok(data)) => {
                        channel.data(data.as_bytes()).await.unwrap()},
                    Some(Err(_)) => unreachable!("Idk how this branch can be reached..."),
                    _ => {
                        debug!("ssh session: client disconnected");
                        break;
                    },
                };
            },
            // There's an event available on the session channel
            Some(msg) = channel.wait() => {
                match msg {
                    // Write data to the terminal
                    ChannelMsg::Data { ref data } => {
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

struct Client {}

#[async_trait]
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
        let ssh_private_key_path = config.ssh.private_key_file.to_owned();
        warp::path!(String / "connect")
            .and(warp::ws())
            .map(move |name: String, ws: ws::Ws| {
                // And then our closure will be called when it completes...
                let store = store.clone();
                let ssh_private_key_path = ssh_private_key_path.to_owned();
                ws.on_upgrade(move |websocket| {
                    let store = store.clone();
                    let ssh_private_key_path = ssh_private_key_path.to_owned();
                    async move {
                        connect(ssh_private_key_path, &name, store, websocket).await;
                    }
                })
            })
    };

    connect
}
