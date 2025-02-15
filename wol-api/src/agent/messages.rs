use serde::{Deserialize, Serialize};

use crate::machine::application::ApplicationInfo;
pub type WebtransportCertificateHash = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHello {
    pub machine_name: String,
    pub applications: Vec<ApplicationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentMessage {
    Hello(AgentHello),
    VdiCertificateHash(WebtransportCertificateHash),
    VdiClosed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    OpenVdi,
}
