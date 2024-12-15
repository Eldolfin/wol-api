use super::service::{State, Store, StoreInner, TIME_BEFORE_ASSUMING_WOL_FAILED};
use crate::config::Config;

use core::convert::Infallible;
use http::status::StatusCode;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use utoipa::OpenApi;
use warp::{
    http,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

const MACHINE_REFRESH_INTERVAL: time::Duration = Duration::from_secs(10);

#[derive(OpenApi)]
#[openapi(paths(list, wake, shutdown))]
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
#[allow(clippy::significant_drop_tightening)]
pub async fn shutdown(
    store: Store,
    name: String,
    dry_run: bool,
) -> Result<Box<dyn Reply>, Infallible> {
    let mut lock = store.lock().await;
    let Some(machine) = lock.by_name_mut(&name) else {
        return Ok(Box::new(reply::with_status(
            "Machine does not exist",
            http::StatusCode::NOT_FOUND,
        )));
    };

    Ok(Box::new(reply::with_status(
        machine.shutdown(dry_run).await,
        StatusCode::OK,
    )))
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
#[allow(clippy::significant_drop_tightening)]
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

pub fn handlers(
    config: &Config,
    dry_run: bool,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let store = Arc::new(Mutex::new(StoreInner::new(config)));

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

    {
        let store = store.clone();
        tokio::spawn(async move {
            loop {
                store.lock().await.refresh_machine_state().await;
                time::sleep(MACHINE_REFRESH_INTERVAL).await;
            }
        });
    }

    list.or(wake).or(shutdown)
}
