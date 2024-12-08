use core::str::FromStr;
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use machine::Store;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;
use std::{collections::HashMap, convert::Infallible, path::PathBuf, sync::Mutex};
use warp::Filter;
use wol::MacAddr;

use anyhow::Context as _;
use clap::Parser;
use log::{debug, info};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// Application configuration
struct Args {
    /// do not actually send wol packets
    #[arg(short = 'n')]
    dry_run: bool,

    /// path to the config file
    #[arg(short = 'c')]
    config_path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct MachineConfig {
    #[schema(example = "192.168.1.4")]
    ip: String,
    #[schema(example = "f4:93:9f:eb:56:a8")]
    mac: String,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
struct Config {
    machines: HashMap<String, MachineConfig>,
}


#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/machine", api = machine::MachineApi)
    ),
    tags(
        (name = "wol", description = "Power on and off computers API")
    )
)]
struct ApiDoc;

mod machine {
    use std::{convert::Infallible, sync::{Arc, Mutex}};

    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, OpenApi, ToSchema};
    use warp::reply::{self, Reply};

    use crate::Config;

    pub type Store = Arc<Mutex<Config>>;

    #[derive(OpenApi)]
    // #[openapi(paths(machine_list, machine_shutdown))]
    #[openapi(paths(list))]
    pub struct MachineApi;

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
            (status = 200, description = "List machines successfully", body = super::Config)
        )
    )]
    pub async fn list(
        store: Store,
    ) -> Result<impl Reply, Infallible> {
        let machines = store.lock().unwrap();
        Ok(reply::json(&machines.clone()))
        }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    debug!("{args:?}");

    let config: Config = Figment::new()
        .merge(Yaml::file(args.config_path))
        .extract()
        .context("Failed to parse config file")?;

    let store = Store::new(Mutex::new(config.clone()));

    let api_doc = warp::path!("api-doc.json").map(|| warp::reply::json(&ApiDoc::openapi()));
    let rapidoc_handler = warp::path("rapidoc")
        .and(warp::get())
        .map(|| warp::reply::html(RapiDoc::new("/api-doc.json").to_html()));


    let machine_list =
        warp::path!("machine"/"list").and(warp::get()).and(with_store(store.clone())).and_then(machine::list);

    let machine_wake = warp::path!("wake" / String).map(move |mac_addr: String| {
        let send_wol = match args.dry_run {
            true => send_wol_dry_run,
            false => send_wol,
        };
        match send_wol(mac_addr.as_str()) {
            Ok(()) => "Ok".to_owned(),
            Err(e) => e.to_string(),
        }
    });

    let routes = warp::get().and(machine_wake.or(machine_list).or(api_doc).or(rapidoc_handler));
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    Ok(())
}
 fn with_store(store: Store) -> impl Filter<Extract = (Store,), Error = Infallible> + Clone {
        warp::any().map(move || store.clone())
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
        "Sending wake on lan to {}",
        mac_addr.to_string().to_uppercase()
    );
    Ok(())
}
