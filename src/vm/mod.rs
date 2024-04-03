use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod eval;

#[derive(Default)]
pub struct VM {
  pub registers: [u32; 32],
  pub pc: usize,
  pub memory: Vec<u8>,
  pub stack: Vec<u8>,
  pub data_variables: HashMap<String, usize>,
  pub statements: Vec<celestial_hub_astrolabe::ast::Statement>,
  pub io_interruption: Option<IOInterruption>,
  pub memory_patch: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum IOInterruption {
  Input(InputRequest),
  Output(String),
  Halt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum InputRequest {
  String,
  Number,
}

pub struct InitVMArgs {
  pub stack_size: usize,
  pub memory_size: usize,
}

impl InitVMArgs {
  pub fn check_memory_size(&self) -> anyhow::Result<()> {
    if !((self.memory_size > 0 && (self.memory_size & (self.memory_size - 1)) == 0)
      && (self.stack_size > 0 && (self.stack_size & (self.stack_size - 1)) == 0))
    {
      anyhow::bail!("memory size and stack size must be a power of 2");
    }

    Ok(())
  }
}

impl Default for InitVMArgs {
  fn default() -> Self {
    InitVMArgs {
      stack_size: 1024 * 1024,
      memory_size: 1024 * 1024,
    }
  }
}

impl VM {
  pub fn new(args: InitVMArgs) -> anyhow::Result<VM> {
    if args.check_memory_size().is_err() {
      anyhow::bail!("memory size and stack size must be a power of 2");
    }

    let InitVMArgs {
      stack_size,
      memory_size,
    } = args;

    Ok(VM {
      memory: vec![0; memory_size],
      stack: vec![0; stack_size],
      ..Default::default()
    })
  }
}
