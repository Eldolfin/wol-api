use directories::ProjectDirs;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_scalar::Scalar;
use warp::{reply, Filter as _};
use wol_relay_server::{
    cache::{self, cache_images},
    config::{self},
    consts::{API_PATH, CONFIG_AUTO_RELOAD},
    machine::{self, service::StoreInner},
};

use clap::Parser;
use log::debug;

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

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/machine", api = machine::api::Api),
        (path = "/cache", api = cache::ImageApi)
    ),
    tags(
        (name = "wol", description = "Power on and off computers API")
    )
)]
struct InnerApiDoc;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = API_PATH, api = InnerApiDoc)
    ),
    tags(
        (name = "wol", description = "Api for wol panel")
    )
)]
struct ApiDoc;

#[tokio::main(flavor = "multi_thread", worker_threads = 29)]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let dirs =
        ProjectDirs::from("top", "eldolfin", "wol-api").expect("to be able to have project dirs");

    let args = Args::parse();
    debug!("{args:?}");

    let (config, mut config_changed) = config::open(&args.config_path, CONFIG_AUTO_RELOAD)?;

    let config_val = config.lock().unwrap().clone();
    *config.lock().unwrap() = cache::cache_images(&dirs, config_val).await?;

    let api_doc = warp::path!("api-doc.json").map(|| reply::json(&ApiDoc::openapi()));
    let rapidoc_handler =
        warp::path!("rapidoc").map(|| reply::html(RapiDoc::new("/api/api-doc.json").to_html()));

    let scalar_handler = warp::path!("doc").map(move || {
        let html = Scalar::new(ApiDoc::openapi())
            .custom_html(include_str!("../res/scalar.html"))
            .to_html();
        reply::html(html)
    });

    let image_cache = cache::image_api(&dirs)?;

    // let cors = warp::cors().allow_origin("http://localhost:3000").allow_methods(vec!["GET", "POST"]);
    // let cors = warp::cors().allow_any_origin().allow_methods(["GET", "POST", "OPTIONS"]);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(["GET", "POST", "OPTIONS"]);

    let listening_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3030);
    println!("Listening on http://{listening_addr}");
    let store = Arc::new(sync::Mutex::new(StoreInner::new(&config.lock().unwrap())?));

    // TODO: this doesn't need to be a select anymore, just spawn each tasks
    loop {
        let (handlers, bg_task) =
            machine::api::handlers(&config.lock().unwrap(), store.clone(), args.dry_run)?;
        let machine_api = warp::path("machine").and(handlers);
        let routes = api_doc
            .or(scalar_handler)
            .or(rapidoc_handler)
            .or(machine_api)
            .or(image_cache.clone())
            .with(&cors);
        let routes = warp::path(API_PATH.strip_prefix("/").unwrap()).and(routes);
        tokio::select! {
            biased;

            v = config_changed.recv() => {
                match cache_images(&dirs, config.lock().unwrap().clone()).await {
                    Ok(cached_config) => *config.lock().unwrap() = cached_config,
                    Err(e) => log::error!("{}", e.context("Failed to cache images")),
                };
                let new_store = StoreInner::new(&config.lock().unwrap())?;
                *store.lock().await = new_store;
                v.unwrap();
            },
            _ = warp::serve(routes).run(listening_addr) => {},
            _ = bg_task => {},
        };
        log::info!("restarting the server");
    }
}
