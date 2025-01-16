use anyhow::Context as _;
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use wol_relay_server::{config::Config, machine::service::*};

#[tokio::test]
async fn machine_wake_shutdown_test_dry_run() -> anyhow::Result<()> {
    const DRY_RUN: bool = true;
    let config: Config = Figment::new()
        .merge(Yaml::string(include_str!("./simple_config.yml")))
        .extract()
        .context("Failed to parse config file")?;
    let mut store = StoreInner::new(&config).context("Could not create store")?;
    let machine = store
        .by_name_mut(config.machines.keys().next().unwrap())
        .unwrap();
    assert_eq!(machine.infos.name, "machine1");
    assert_eq!(machine.infos.config.mac, "02:42:ac:12:00:02");
    assert_eq!(machine.infos.config.ip, "127.0.0.1:2222");
    assert_eq!(
        machine.infos.state,
        State::Unknown,
        "The machine should start with an unknown state"
    );
    machine
        .wake(DRY_RUN)
        .expect("failed to wake the machine in dry_run mode");

    assert_eq!(
        machine.infos.state,
        State::PendingOn,
        "Sending a wake on lan should put the machine in PendingOn"
    );
    machine.update_state().await;
    assert_eq!(
        machine.infos.state,
        State::On,
        "The computer running the tests should be on and reachable"
    );

    machine.shutdown(DRY_RUN).await;

    assert_eq!(
        machine.infos.state,
        State::PendingOff,
        "Shutdown should turn the machine.state Off"
    );

    machine.update_state().await;
    assert_eq!(machine.infos.state, State::PendingOff,);

    Ok(())
}
