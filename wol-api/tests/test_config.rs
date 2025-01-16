use std::{
    fs::{self, File},
    io::Write as _,
    time::Duration,
};

use anyhow::{Context as _, Result};
use figment::{
    providers::{Format as _, Yaml},
    Figment,
};
use rstest::{fixture, rstest};
use tempfile::TempDir;
use tokio::time::timeout;
use wol_relay_server::config::{self, Config};
use wol_relay_server::test;

#[fixture]
fn test_config() -> Config {
    test::logfxt();
    let config_str = include_str!("./simple_config.yml");
    Figment::new()
        .merge(Yaml::string(config_str))
        .extract()
        .with_context(|| format!("Failed to parse test static config file: {config_str}"))
        .unwrap()
}

#[rstest]
#[tokio::test]
async fn config_reload(mut test_config: Config) -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &test_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let (in_file_config, mut config_changed) = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        test_config,
        "Config differs before being changed"
    );

    test_config.machines.get_mut("machine1").unwrap().ip = "192.168.1.1".into();

    let config_file = File::create(&config_filename).context("Could not re open config file")?;

    serde_yaml::to_writer(&config_file, &test_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    // wait for the config be reloaded
    timeout(Duration::from_millis(10), config_changed.recv())
        .await
        .context("expected config_changed to be fired")?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        test_config,
        "Config differs after being changed"
    );

    Ok(())
}

#[tokio::test]
async fn config_reload_multiple_times() -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let mut in_memory_config = test_config();

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let (in_file_config, mut config_changed) = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs before being changed"
    );

    for i in 0i32..10i32 {
        in_memory_config.machines.get_mut("machine1").unwrap().ip = format!("127.0.0.1:{i}");

        let config_file =
            File::create(&config_filename).context("Could not re open config file")?;

        serde_yaml::to_writer(&config_file, &in_memory_config)
            .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

        // wait for the config be reloaded
        timeout(Duration::from_millis(10), config_changed.recv())
            .await
            .context("expected config_changed to be fired")?;

        assert_eq!(
            in_file_config.lock().unwrap().clone(),
            in_memory_config,
            "Config differs after being changed"
        );
    }

    Ok(())
}

#[tokio::test]
async fn config_reload_error() -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let in_memory_config = test_config();

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let (in_file_config, mut config_changed) = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs before being changed"
    );

    let mut in_memory_config_modified = in_memory_config.clone();
    in_memory_config_modified
        .machines
        .get_mut("machine1")
        .unwrap()
        .ip = "192.168.1.1".into();

    let config_file = File::create(&config_filename).context("Could not re open config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config_modified)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;
    write!(&config_file, "      - random_field: false").unwrap();

    // wait for the config be reloaded
    timeout(Duration::from_millis(10), config_changed.recv())
        .await
        .expect_err("expected config_changed to not be fired");

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config should not be changed if there was an error loading it"
    );

    Ok(())
}

#[tokio::test]
async fn config_not_reloading_if_sibbling_changed() -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let mut in_memory_config = test_config();

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let (in_file_config, mut config_changed) = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs before being changed"
    );

    in_memory_config.machines.get_mut("machine1").unwrap().ip = "192.168.1.1".into();

    let config_file = File::create(dir.path().join("wol-config2.yml"))
        .context("Could not re open config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    // wait for the config be reloaded
    timeout(Duration::from_millis(10), config_changed.recv())
        .await
        .expect_err("expected config_changed to not be fired");

    Ok(())
}

#[tokio::test]
async fn config_reload_edit_like_editor() -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let mut in_memory_config = test_config();

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let (in_file_config, mut config_changed) = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs before being changed"
    );

    in_memory_config.machines.get_mut("machine1").unwrap().ip = "192.168.1.1".into();

    fs::remove_file(&config_filename)?;
    let config_file =
        File::create_new(&config_filename).context("Could not re open config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    // wait for the config be reloaded
    timeout(Duration::from_millis(10), config_changed.recv())
        .await
        .context("expected config_changed to be fired")?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs after being changed"
    );

    Ok(())
}
