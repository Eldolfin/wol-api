pub mod responses;
use super::service::{recv_agent_msg, State, Store, Task};
use crate::{
    agent::messages::AgentMessage,
    config::Config,
    consts::{MACHINE_REFRESH_INTERVAL, SEND_STATE_INTERVAL, TIME_BEFORE_ASSUMING_WOL_FAILED},
    machine::ssh,
};
use responses::{ListMachineResponse, OpenVdiError};
use urlencoding;

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
    paths(list, wake, shutdown, open_vdi, task, list_ws, agent, open_application),
    nest(
        (path = "/ssh", api = ssh::api::Api)
    ),
)]
pub struct Api;

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List machines successfully", body = ListMachineResponse)
    )
)]
pub async fn list(store: Store) -> Result<Box<dyn Reply>, Infallible> {
    let machines = store.lock().await;
    Ok(Box::new(reply::json(&ListMachineResponse::from(
        &machines.machines,
    ))))
}

#[utoipa::path(
    get,
    path = "/list_ws",
    responses(
        (status = 101, description = "Switching protocol to websocket", body = ListMachineResponse)
    )
)]
pub async fn list_ws(store: Store, websocket: WebSocket) {
    let (mut tx, _rx) = websocket.split();
    let mut last_machines_states = None;
    loop {
        let machines = ListMachineResponse::from(&store.lock().await.machines);
        if Some(&machines) != last_machines_states.as_ref() {
            let to_string = serde_json::to_string(&machines).unwrap();
            last_machines_states = Some(machines);
            let res = tx.send(Message::text(to_string)).await;
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
        (status = 101, description = "Switching protocol to websocket")
    )
)]
pub async fn agent(store: Store, mut websocket: WebSocket) {
    let agent_hello = match recv_agent_msg(&mut websocket).await {
        Ok(res) => res,
        Err(err) => {
            error!("Could not receive agent hello: {:#}", err);
            return;
        }
    };
    let agent_hello = match agent_hello {
        AgentMessage::Hello(agent_hello) => agent_hello,
        msg => {
            error!(
                "Agent didn't send a hello as a first message, he sent {:?} instead",
                msg
            );
            return;
        }
    };

    let mut lock = store.lock().await;
    if let Some(machine) = lock.by_name_mut(&agent_hello.machine_name) {
        machine.set_applications(agent_hello.applications).await;
        machine.set_connection(websocket);
        debug!(
            "Machine `{}` successfully sent its list of applications",
            machine.infos.name
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
    path = "/{name}/open_vdi",
    responses(
        // TODO: send serverCertificateHash as a response
        (status = 200, description = "Opened the vdi successfully"),
        (status = 404, description = "Machine does not exist"),
        (status = 500, description = "Failed to open vdi", body = OpenVdiError)
    ),
    params(
        ("name" = String, Path, description = "Name of the machine on which to open the vdi")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn open_vdi(store: Store, name: String) -> Result<Box<dyn Reply>, Infallible> {
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(Box::new(reply::with_status(
            reply::Response::default(),
            http::StatusCode::NOT_FOUND,
        )));
    };

    match machine.open_vdi().await {
        Ok(()) => Ok(Box::new(reply::with_status(
            reply::Response::default(),
            StatusCode::OK,
        ))),
        Err(err) => Ok(Box::new(reply::with_status(
            serde_json::to_string(&err).unwrap(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
    }
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
    path = "/{name}/open_application/{application_name}",
    responses(
        (status = 200, description = "Application opened successfully"),
    ),
    params(
        ("name" = String, Path, description = "Name of the machine"),
        ("application_name" = String, Path, description = "Name of the application")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn open_application(
    store: Store,
    name: String,
    application_name: String,
    dry_run: bool,
) -> Result<impl Reply, Infallible> {
    let application_name = urlencoding::decode(&application_name).unwrap();
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(reply::with_status(
            "Machine does not exist".to_owned(),
            http::StatusCode::NOT_FOUND,
        ));
    };
    match machine.open_app(&application_name, dry_run).await {
        Ok(()) => Ok(reply::with_status("Success".to_owned(), StatusCode::OK)),
        Err(msg) => Ok(reply::with_status(
            format!("{msg:#}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
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
            if machine.infos.state == State::PendingOn {
                machine.infos.state = State::Off;
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
    let open_vdi = {
        let store = store.clone();
        warp::path!(String / "open_vdi").and_then(move |name: String| open_vdi(store.clone(), name))
    };
    let task = {
        let store = store.clone();
        warp::path!(String / "task")
            .and(json())
            .and_then(move |name, body: Task| task(store.clone(), name, dry_run, body))
    };
    let open_application = {
        let store = store.clone();
        warp::path!(String / "open_application" / String).and_then(move |name, application_name| {
            open_application(store.clone(), name, application_name, dry_run)
        })
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
        .or(open_vdi)
        .or(task)
        .or(list_ws)
        .or(ssh_handlers)
        .or(agent)
        .or(open_application);

    Ok((routes, check_state_thread))
}
