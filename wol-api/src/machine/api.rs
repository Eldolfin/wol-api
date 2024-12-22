use super::service::{State, Store, StoreInner, Task};
use crate::{
    config::Config,
    consts::{MACHINE_REFRESH_INTERVAL, TIME_BEFORE_ASSUMING_WOL_FAILED},
};

use core::convert::Infallible;
use http::status::StatusCode;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::{sync::Mutex, time};
use utoipa::OpenApi;
use warp::{
    body::json,
    http,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

#[derive(OpenApi)]
#[openapi(paths(list, wake, shutdown, task))]
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
        (status = 200, description = "Task added to the queue successfully"),
    ),
    request_body = Task,
    params(
        ("name" = String, Path, description = "Name of the machine to run the task on")
    ),
)]
#[expect(clippy::significant_drop_tightening, reason = "todo fix mais flemme")]
pub async fn task(
    store: Arc<Mutex<StoreInner>>,
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
    // TODO: change machine state
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
    dry_run: bool,
) -> anyhow::Result<(
    impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone,
    Pin<Box<dyn Future<Output = ()>>>,
)> {
    let store = Arc::new(Mutex::new(StoreInner::new(config)?));

    let list = {
        let store = store.clone();
        warp::path!("list")
            .and(warp::get())
            .and_then(move || list(store.clone()))
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

    Ok((list.or(wake).or(shutdown).or(task), check_state_thread))
}
