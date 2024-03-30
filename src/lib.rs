use serde::{Deserialize, Serialize};

pub mod vm;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum VMCommand {
  Step,
  Run,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CommandMessage {
  pub command: VMCommand,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatusUpdateMessage {
  pub registers: Vec<u32>,
  pub pc: usize,
  pub io_interruption: Option<vm::IOInterruption>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorMessage {
  pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum SocketMessage {
  Command(CommandMessage),
  Input(String),
  StatusUpdate(StatusUpdateMessage),
  Error(ErrorMessage),
}
