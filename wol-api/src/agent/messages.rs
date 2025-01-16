use serde::{Deserialize, Serialize};

use crate::machine::application::ApplicationInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHello {
    pub machine_name: String,
    pub applications: Vec<ApplicationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentMessage {
    Hello(AgentHello),
    VdiClosed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    OpenVdi,
}
