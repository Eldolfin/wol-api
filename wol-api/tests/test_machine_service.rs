use anyhow::Context as _;
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use wol_relay_server::{config::Config, machine::service::*};

#[tokio::test]
async fn machine_wake_shutdown_test() -> anyhow::Result<()> {
    let config: Config = Figment::new()
        .merge(Yaml::string(include_str!("./simple_config.yml")))
        .extract()
        .context("Failed to parse config file")?;
    let mut store = StoreInner::new(&config);
    let machine = store
        .by_name_mut(config.machines.keys().next().unwrap())
        .unwrap();
    assert!(machine.name == "tour");
    assert!(machine.config.mac == "f4:93:9f:eb:56:a8");
    assert!(machine.config.ip == "127.0.0.1");
    assert!(machine.config.ssh_port == 22);
    assert!(
        machine.state == State::Unknown,
        "The machine should start with an unknown state"
    );
    machine
        .wake(true)
        .expect("failed to wake the machine in dry_run mode");

    assert!(
        machine.state == State::PendingOn,
        "Sending a wake on lan should put the machine in PendingOn"
    );
    machine.update_state().await;
    assert!(
        machine.state == State::On,
        "The computer running the tests should be on and reachable"
    );

    machine.shutdown(true).await;

    assert!(
        machine.state == State::Off,
        "Shutdown should turn the machine.state Off"
    );

    machine.update_state().await;
    assert!(
        machine.state == State::On,
        "This is a temporary assert because update_state is not mocked"
    ); // TODO: mock, it should still be Off here

    Ok(())
}
