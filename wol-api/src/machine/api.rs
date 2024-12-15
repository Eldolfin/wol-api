use super::{wol, Store};
use crate::{config::Config, machine::StoreInner};

use core::convert::Infallible;
use log::{debug, info};
use std::{sync::Arc, time::Duration};
use tokio::process::Command;
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
        (status = 200, description = "List machines successfully", body = Config)
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
pub async fn shutdown(
    store: Store,
    name: String,
    dry_run: bool,
) -> Result<Box<dyn Reply>, Infallible> {
    let Some(machine) = store.lock().await.by_name(&name) else {
        return Ok(Box::new(reply::with_status(
            "Machine does not exist",
            http::StatusCode::NOT_FOUND,
        )));
    };
    let mut cmd = Command::new("ssh");
    cmd.arg("-i")
        .arg("~/.ssh/id_ed25519")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg(format!("oscar@{}", machine.config.ip))
        .arg("sudo")
        .arg("systemctl")
        .arg("poweroff");
    info!("Shutting down machine '{}'", name);
    debug!(
        "Running command: {:?}{}",
        &cmd,
        if dry_run { " (dry run)" } else { "" }
    );
    if !dry_run {
        let output = cmd.output().await;
        if let Err(err) = output {
            return Ok(Box::new(reply::with_status(
                format!("ssh command failed: {err}"),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )));
        };
        debug!("Command output: {:?}", &output);
    }
    Ok(Box::new(reply::reply()))
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
pub async fn wake(store: Store, name: String, dry_run: bool) -> Result<Box<dyn Reply>, Infallible> {
    // TODO: change machine state
    let Some(machine) = store.lock().await.by_name(&name) else {
        return Ok(Box::new(reply::with_status(
            "Machine does not exist",
            http::StatusCode::NOT_FOUND,
        )));
    };
    match wol::send(&machine.config.mac, dry_run) {
        Ok(()) => Ok(Box::new(reply::reply())),
        Err(e) => Ok(Box::new(e.to_string())),
    }
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
