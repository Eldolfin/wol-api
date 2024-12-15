use anyhow::Context as _;
use core::convert::Infallible;
use core::str::FromStr;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use wol::MacAddr;

use utoipa::OpenApi;
use warp::{
    http,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

use crate::config::Config;

pub type Store = Arc<Mutex<Config>>;

#[derive(OpenApi)]
#[openapi(paths(list, wake, shutdown))]
pub struct Api;

// #[derive(Serialize, Deserialize, ToSchema)]
// #[serde(rename_all = "snake_case")]
// pub struct Machine {
//     config: super::MachineConfig,
//     state: MachineState,
// }

// #[derive(Serialize, Deserialize, ToSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum MachineState {
//     On,
//     Off
// }

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List machines successfully", body = Config)
    )
)]
pub fn list(store: Store) -> impl warp::Reply {
    let machines = store.lock().unwrap();
    reply::json(&machines.clone())
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
    _dry_run: bool,
) -> Result<Box<dyn Reply>, Infallible> {
    let Some(machine) = store.lock().unwrap().by_name(&name) else {
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
        .arg(format!("oscar@{}", machine.ip))
        .arg("sudo")
        .arg("systemctl")
        .arg("poweroff");
    debug!("Running command: {:?}", &cmd);
    let output = cmd.output().await;
    if let Err(err) = output {
        return Ok(Box::new(reply::with_status(
            format!("ssh command failed: {err}"),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )));
    };
    debug!("Command output: {:?}", &output);
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
pub fn wake(store: Store, name: &str, dry_run: bool) -> Result<Box<dyn Reply>, Infallible> {
    // TODO: change machine state
    let Some(machine) = store.lock().unwrap().by_name(name) else {
        return Ok(Box::new(reply::with_status(
            "Machine does not exist",
            http::StatusCode::NOT_FOUND,
        )));
    };
    let send_wol = match dry_run {
        true => send_wol_dry_run,
        false => send_wol,
    };
    match send_wol(&machine.mac) {
        Ok(()) => Ok(Box::new(reply::reply())),
        Err(e) => Ok(Box::new(e.to_string())),
    }
}

pub fn handlers(
    config: &Config,
    dry_run: bool,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    fn with_store(store: Store) -> impl Filter<Extract = (Store,), Error = Infallible> + Clone {
        warp::any().map(move || store.clone())
    }
    let store = Store::new(Mutex::new(config.clone()));

    let list = warp::path!("list")
        .and(warp::get())
        .and(with_store(store.clone()))
        .map(list);

    let wake = {
        let store = store.clone();
        warp::path!(String / "wake")
            .map(move |mac_addr: String| wake(store.clone(), &mac_addr, dry_run))
    };
    let shutdown = {
        let store = store.clone();
        warp::path!(String / "shutdown")
            .and_then(move |mac_addr: String| shutdown(store.clone(), mac_addr, dry_run))
    };

    list.or(wake).or(shutdown)
}
fn send_wol(mac_addr: &str) -> anyhow::Result<()> {
    use wol::send_wol;
    let mac_addr =
        MacAddr::from_str(mac_addr).map_err(|err| anyhow::Error::msg(err.to_string()))?;
    info!(
        "Sending wake on lan to {}",
        mac_addr.to_string().to_uppercase()
    );
    send_wol(mac_addr, None, None).context("Could not send wold")?;
    Ok(())
}

fn send_wol_dry_run(mac_addr: &str) -> anyhow::Result<()> {
    let mac_addr =
        MacAddr::from_str(mac_addr).map_err(|err| anyhow::Error::msg(err.to_string()))?;
    info!(
        "Sending wake on lan to {} (dry run)",
        mac_addr.to_string().to_uppercase()
    );
    Ok(())
}
