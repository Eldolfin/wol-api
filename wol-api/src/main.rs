use warp::Filter;
use wol::MacAddr;
use core::str::FromStr;

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
}


#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    debug!("{args:?}");

       let hello = warp::path!("wake" / String)
        .map(move |mac_addr: String| {
            let send_wol = match args.dry_run {
                true => send_wol_dry_run,
                false => send_wol,
            };
            match send_wol(mac_addr.as_str()) {
                Ok(()) => "Ok".to_owned(),
                Err(e) => e.to_string(),
            }
        });

    warp::serve(hello)
        .run(([0, 0, 0, 0], 3030))
        .await;

}

fn send_wol(mac_addr: &str) -> anyhow::Result<()>{
use wol::send_wol;
    let mac_addr = MacAddr::from_str(mac_addr).map_err(|err| anyhow::Error::msg(err.to_string()))?;
    info!("Sending wake on lan to {}",mac_addr.to_string().to_uppercase());
    send_wol(
        mac_addr,
        None,
        None,
    )
    .context("Could not send wold")?;
    Ok(())
    }

fn send_wol_dry_run(mac_addr: &str) -> anyhow::Result<()>{
    let mac_addr = MacAddr::from_str(mac_addr).map_err(|err| anyhow::Error::msg(err.to_string()))?;
    info!("Sending wake on lan to {}",mac_addr.to_string().to_uppercase());
    Ok(())
}
