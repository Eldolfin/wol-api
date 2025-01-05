use serde::{Deserialize, Serialize};

use crate::machine::application::ApplicationInfo;

#[derive(Serialize, Deserialize)]
pub struct AgentHello {
    pub machine_name: String,
    pub applications: Vec<ApplicationInfo>,
}
