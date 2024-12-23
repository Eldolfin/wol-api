use futures_util::{FutureExt as _, StreamExt as _};
use utoipa::OpenApi;
use warp::{
    filters::ws::{self, WebSocket},
    reject::Rejection,
    reply::Reply,
    Filter,
};

use crate::machine::service::Store;

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
async fn connect(machine_name: &str, store: Store, websocket: WebSocket) {
    let (tx, rx) = websocket.split();
    // Just echo all messages back...
    rx.forward(tx)
        .map(|result| {
            if let Err(e) = result {
                eprintln!("websocket error: {:?}", e);
            }
        })
        .await

    // loop {
    //     let machines = store.lock().await.to_owned();
    //     tx.send(Message::text(serde_json::to_string(&machines).unwrap()))
    //         .await
    //         .expect("Failed to send to websocket");
    // }

    // let mut ssh = Session::connect(
    //     todo!("private key field in config file"),
    //     cli.username.unwrap_or("root".to_string()),
    //     cli.openssh_certificate,
    //     (cli.host, cli.port),
    // )
    // .await?;
}

#[expect(clippy::let_and_return, reason = "might add more endpoints here later")]
pub fn handlers(store: Store) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let connect = {
        let store = store.clone();
        warp::path!(String / "connect")
            .and(warp::ws())
            .map(move |name: String, ws: ws::Ws| {
                // And then our closure will be called when it completes...
                let store = store.clone();
                ws.on_upgrade(move |websocket| {
                    let store = store.clone();
                    async move {
                        connect(&name, store, websocket).await;
                    }
                })
            })
    };

    connect
}
