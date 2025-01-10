use super::service::{State, Store, StoreInner, Task};
use crate::{
    agent::messages::AgentHello,
    config::Config,
    consts::{MACHINE_REFRESH_INTERVAL, SEND_STATE_INTERVAL, TIME_BEFORE_ASSUMING_WOL_FAILED},
    machine::ssh,
};

use core::convert::Infallible;
use futures_util::{SinkExt as _, StreamExt as _};
use http::status::StatusCode;
use log::{debug, error};
use std::{future::Future, pin::Pin};
use tokio::time;
use utoipa::OpenApi;
use warp::{
    body::json,
    filters::ws::{Message, WebSocket},
    http,
    reject::Rejection,
    reply::{self, Reply},
    ws, Filter,
};

#[derive(OpenApi)]
#[openapi(
    paths(list, wake, shutdown, task, list_ws, agent),
    nest(
        (path = "/ssh", api = ssh::api::Api)
    ),
)]
pub struct Api;

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List machines successfully", body = StoreInner)
    )
)]
pub async fn list(store: Store) -> Result<Box<dyn Reply>, Infallible> {
    let machines = store.lock().await;
    Ok(Box::new(reply::json(&machines.clone())))
}

#[utoipa::path(
    get,
    path = "/list_ws",
    responses(
        (status = 101, description = "Switching protocol to websocket", body = StoreInner)
    )
)]
pub async fn list_ws(store: Store, websocket: WebSocket) {
    let (mut tx, _rx) = websocket.split();
    let mut last_machines_states = None;
    loop {
        let machines = store.lock().await.to_owned();
        if Some(machines.clone()) != last_machines_states {
            last_machines_states = Some(machines.clone());
            let res = tx
                .send(Message::text(serde_json::to_string(&machines).unwrap()))
                .await;
            match res {
                Ok(_) => (),
                Err(e) => {
                    debug!("/list_ws was closed by the client: {e:#}");
                    break;
                }
            }
        }
        time::sleep(SEND_STATE_INTERVAL).await;
    }
}

#[utoipa::path(
    get,
    path = "/agent",
    responses(
        (status = 101, description = "Switching protocol to websocket", body = StoreInner)
    )
)]
pub async fn agent(store: Store, websocket: WebSocket) {
    let (_tx, mut rx) = websocket.split();
    let agent_hello_msg = match rx.next().await {
        Some(Ok(msg)) => msg,
        Some(Err(err)) => {
            error!("Failed to received agent hello: {:#}", err);
            return;
        }
        _ => {
            error!("Failed to receive agent hello");
            return;
        }
    };
    let Ok(msg_str) = agent_hello_msg.to_str() else {
        error!("Agent sent a message that was not a string");
        return;
    };
    let agent_hello: AgentHello = match serde_json::from_str(msg_str) {
        Ok(hello) => hello,
        Err(err) => {
            error!("Agent sent an incorrect formatted hello message: {:#}", err);
            return;
        }
    };

    let mut lock = store.lock().await;
    if let Some(machine) = lock.by_name_mut(&agent_hello.machine_name) {
        machine.set_applications(agent_hello.applications);
        debug!(
            "Machine `{}` successfully sent its list of applications",
            machine.name
        );
    } else {
        error!(
            "An unknown agent sent a hello message: {}",
            agent_hello.machine_name
        );
    }
}

#[utoipa::path(
    post,
    path = "/{name}/shutdown",
    responses(
        (status = 200, description = "Shutdown the machine successfully"),
        (status = 404, description = "Machine does not exist")
    ),
    params(
        ("name" = String, Path, description = "Name of the machine to shutdown")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn shutdown(store: Store, name: String, dry_run: bool) -> Result<impl Reply, Infallible> {
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(reply::with_status(
            "Machine does not exist".to_owned(),
            http::StatusCode::NOT_FOUND,
        ));
    };

    Ok(reply::with_status(
        machine.shutdown(dry_run).await,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    post,
    path = "/{name}/task",
    responses(
        (status = 200, description = "Task added to the queue successfully", example = "Pushed task 'example task name' successfully"),
    ),
    request_body = Task,
    params(
        ("name" = String, Path, description = "Name of the machine to run the task on")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn task(
    store: Store,
    name: String,
    dry_run: bool,
    task: Task,
) -> Result<impl Reply, Infallible> {
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(reply::with_status(
            "Machine does not exist".to_owned(),
            http::StatusCode::NOT_FOUND,
        ));
    };
    match machine.push_task(task, dry_run) {
        Ok(msg) => Ok(reply::with_status(msg, StatusCode::OK)),
        Err(msg) => Ok(reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

#[utoipa::path(
    post,
    path = "/{name}/wake",
    responses(
        (status = 200, description = "Woke the machine successfully"),
        (status = 404, description = "Machine does not exist")
    ),
    params(
        ("name" = String, Path, description = "Name of the machine to wake")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn wake(store: Store, name: String, dry_run: bool) -> Result<Box<dyn Reply>, Infallible> {
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(Box::new(reply::with_status(
            "Machine does not exist",
            http::StatusCode::NOT_FOUND,
        )));
    };
    {
        let store = store.clone();
        tokio::spawn(async move {
            time::sleep(TIME_BEFORE_ASSUMING_WOL_FAILED).await;
            let mut lock = store.lock().await;
            let machine = lock.by_name_mut(&name).unwrap();
            if machine.state == State::PendingOn {
                machine.state = State::Off;
            }
        });
    }

    Ok(Box::new(match machine.wake(dry_run) {
        Ok(msg) => reply::with_status(msg, StatusCode::OK),
        Err(msg) => reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR),
    }))
}

#[expect(clippy::type_complexity, reason = "aie aie aie")]
pub fn handlers(
    config: &Config,
    store: Store,
    dry_run: bool,
) -> anyhow::Result<(
    impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone,
    Pin<Box<dyn Future<Output = ()>>>,
)> {
    let list = {
        let store = store.clone();
        warp::path!("list")
            .and(warp::get())
            .and_then(move || list(store.clone()))
    };
    let list_ws = {
        let store = store.clone();
        warp::path!("list_ws").and(ws()).map(move |ws: ws::Ws| {
            let store = store.clone();
            ws.on_upgrade(move |websocket| {
                let store = store.clone();
                async move {
                    list_ws(store, websocket).await;
                }
            })
        })
    };
    let agent = {
        let store = store.clone();
        warp::path!("agent").and(ws()).map(move |ws: ws::Ws| {
            let store = store.clone();
            ws.max_message_size(1024 << 20) // 1GB
                .max_frame_size(1024 << 20) // 1GB
                .on_upgrade(move |websocket| {
                    let store = store.clone();
                    async move {
                        agent(store, websocket).await;
                    }
                })
        })
    };

    let wake = {
        let store = store.clone();
        warp::path!(String / "wake")
            .and_then(move |name: String| wake(store.clone(), name, dry_run))
    };
    let shutdown = {
        let store = store.clone();
        warp::path!(String / "shutdown")
            .and_then(move |name: String| shutdown(store.clone(), name, dry_run))
    };
    let task = {
        let store = store.clone();
        warp::path!(String / "task")
            .and(json())
            .and_then(move |name, body: Task| task(store.clone(), name, dry_run, body))
    };

    let check_state_thread = {
        let store = store.clone();
        Box::pin(async move {
            loop {
                store.lock().await.refresh_machine_state().await;
                time::sleep(MACHINE_REFRESH_INTERVAL).await;
            }
        })
    };

    let ssh_handlers = warp::path("ssh").and(ssh::api::handlers(config, store.clone()));

    let routes = list
        .or(wake)
        .or(shutdown)
        .or(task)
        .or(list_ws)
        .or(ssh_handlers)
        .or(agent);

    Ok((routes, check_state_thread))
}
