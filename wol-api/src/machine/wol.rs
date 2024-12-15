use anyhow::Context as _;
use core::str::FromStr;
use log::info;
use wol::MacAddr;

pub fn send(mac_addr: &str, dry_run: bool) -> anyhow::Result<()> {
    use wol::send_wol;
    let mac_addr =
        MacAddr::from_str(mac_addr).map_err(|err| anyhow::Error::msg(err.to_string()))?;
    info!(
        "Sending wake on lan to {}",
        mac_addr.to_string().to_uppercase()
    );
    if !dry_run {
        send_wol(mac_addr, None, None).context("Could not send wold")?;
    }
    Ok(())
}
