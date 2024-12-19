use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use warp::{reply, Filter};
use wol_relay_server::{config::Config, consts::API_PATH, machine};

use anyhow::Context as _;
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
        (path = "/machine", api = machine::api::Api)
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    debug!("{args:?}");

    let config: Config = Figment::new()
        .merge(Yaml::file(&args.config_path))
        .extract()
        .context("Failed to parse config file")?;

    debug!("{config:?}");

    let api_doc = warp::path!("api-doc.json").map(|| reply::json(&ApiDoc::openapi()));
    let rapidoc_handler = warp::path("rapidoc")
        .and(warp::get())
        .map(|| reply::html(RapiDoc::new("/api/api-doc.json").to_html()));

    let machine_api = warp::path("machine").and(machine::api::handlers(&config, args.dry_run)?);

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

    let routes = api_doc.or(rapidoc_handler).or(machine_api).with(cors);
    let routes = warp::path(API_PATH.strip_prefix("/").unwrap()).and(routes);

    let listening_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3030);
    println!("Listening on http://{listening_addr}");
    warp::serve(routes).run(listening_addr).await;
    Ok(())
}
