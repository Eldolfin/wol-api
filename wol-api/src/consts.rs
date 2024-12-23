use std::time::Duration;

pub const API_PATH: &str = "/api";
pub const MACHINE_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
pub const TIME_BEFORE_ASSUMING_WOL_FAILED: Duration = Duration::from_secs(60);
pub const CONFIG_AUTO_RELOAD: bool = true;
pub const SEND_STATE_INTERVAL: Duration = Duration::from_millis(100);
