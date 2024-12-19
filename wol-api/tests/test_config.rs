use std::{any::Any, collections::HashMap, fs::File, io};

use anyhow::{Context, Result};
use tempfile::TempDir;
use wol_relay_server::config::{self, Config, MachineCfg};

#[tokio::test]
async fn test_config_reload() -> Result<()> {
    const AUTO_RELOAD: bool = true;

    let dir = TempDir::new()?;

    let config_filename = dir.path().join("wol-config.yml");

    let mut in_memory_config = Config {
        machines: HashMap::from([(
            "machine1".into(),
            MachineCfg {
                ip: "192.168.1.167".into(),
                mac: "f4:93:9f:eb:56:a8".into(),
                ssh_port: 22,
                tasks: vec![],
            },
        )]),
    };

    let config_file = File::create_new(&config_filename).context("Could not create config file")?;

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;

    let in_file_config = config::open(&config_filename, AUTO_RELOAD)?;

    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs before being changed"
    );

    in_memory_config.machines.get_mut("machine1").unwrap().ip = "192.168.1.1".into();

    serde_yaml::to_writer(&config_file, &in_memory_config)
        .with_context(|| format!("Failed to write to {}", config_filename.display()))?;
    assert_eq!(
        in_file_config.lock().unwrap().clone(),
        in_memory_config,
        "Config differs after being changed"
    );

    Ok(())
}
