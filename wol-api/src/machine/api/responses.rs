use serde::Serialize;
use utoipa::ToSchema;

use crate::machine::service::{Machine, MachineInfos};

#[derive(Serialize, ToSchema, PartialEq, Eq)]
pub struct ListMachineResponse {
    machines: Vec<MachineInfos>,
}

impl From<&Vec<Machine>> for ListMachineResponse {
    fn from(value: &Vec<Machine>) -> Self {
        Self {
            machines: value.iter().map(|machine| machine.infos.clone()).collect(),
        }
    }
}

#[derive(Serialize, ToSchema, PartialEq, Eq)]
pub enum AgentComunicationError {
    NotConnected,
    SendFailed(String),
}

#[derive(Serialize, ToSchema, PartialEq, Eq)]
pub enum OpenVdiError {
    AgentComunicationError(AgentComunicationError),
    AlreadyOpened,
}
