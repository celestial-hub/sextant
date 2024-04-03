use anyhow::Context;
use celestial_hub_astrolabe::ast::{Instruction, InstructionArgument, Program, Statement};

use crate::{
  vm::eval::utils::{memory_map::Memory, register::RegisterPosition},
  StatusUpdateMessage,
};

use self::utils::register::get_register_position;

use super::{IOInterruption, InputRequest, VM};
pub mod utils;

impl VM {
  pub fn eval(&mut self, program: Program) -> anyhow::Result<()> {
    self.load(program).context("Load program failed")?;

    while self.pc < self.statements.len() {
      let statement = self.statements[self.pc].clone();
      self.pc += 1;

      match statement {
        Statement::Label(_) => {}
        Statement::Instruction(instruction) => {
          self.eval_instruction(&instruction)?;
        }
      }
    }

    Ok(())
  }

  pub fn eval_instruction(&mut self, instruction: &Instruction) -> anyhow::Result<()> {
    match &instruction {
      Instruction::Li(args) => {
        let (register, immediate) = match (&args[0], &args[1]) {
          (InstructionArgument::Register(register), InstructionArgument::Immediate(immediate)) => {
            (register, immediate)
          }
          _ => {
            eprintln!(
              "Unhandled combination of arguments for li instruction: {:#?}",
              args
            );
            unreachable!()
          }
        };

        self.registers[register.get_register_position()?] = *immediate;

        Ok(())
      }
      Instruction::La(args) => {
        let (register, label) = match (&args[0], &args[1]) {
          (InstructionArgument::Register(register), InstructionArgument::Label(label)) => {
            (register, label)
          }
          _ => {
            eprintln!(
              "Unhandled combination of arguments for la instruction: {:#?}",
              args
            );
            unreachable!()
          }
        };

        let address = self.data_variables.get(label).unwrap();

        self.registers[register.get_register_position()?] = *address as u32;

        Ok(())
      }
      Instruction::Syscall => self.handle_syscall(self.registers[get_register_position("$v0")?]),
      Instruction::Move(args) => {
        let (destination, source) = match (&args[0], &args[1]) {
          (InstructionArgument::Register(register), InstructionArgument::Register(register1)) => {
            (register, register1)
          }
          _ => unreachable!(), // Since it's guaranteed
        };

        self.registers[destination.get_register_position()?] =
          self.registers[source.get_register_position()?];

        Ok(())
      }
      Instruction::Jal(_) => todo!(),
      Instruction::Beq(_) => todo!(),
      Instruction::Sub(_) => todo!(),
      Instruction::Add(args) => {
        let (register, register1, register2) = match (&args[0], &args[1], &args[2]) {
          (
            InstructionArgument::Register(register),
            InstructionArgument::Register(register1),
            InstructionArgument::Register(register2),
          ) => (register, register1, register2),
          _ => unreachable!(), // Since it's guaranteed
        };

        self.registers[register.get_register_position()?] = self.registers
          [register1.get_register_position()?]
          + self.registers[register2.get_register_position()?];

        Ok(())
      }
      Instruction::Jr(_) => todo!(),
      Instruction::Addi(_) => todo!(),
      Instruction::Andi(_) => todo!(),
      Instruction::J(_) => todo!(),
    }
  }

  fn handle_syscall(&mut self, code: u32) -> anyhow::Result<()> {
    match code {
      1 => {
        let value = self.registers[get_register_position("$a0")?];
        self.io_interruption = Some(IOInterruption::Output(value.to_string()))
      }
      4 => {
        let address = self.registers[get_register_position("$a0")?] as usize;
        let mut i = address as usize;
        let mut output = String::new();
        while self.memory[i] != 0 {
          output.push(self.memory[i] as char);
          i += 1;
        }
        self.io_interruption = Some(IOInterruption::Output(output))
      }
      5 => self.io_interruption = Some(IOInterruption::Input(InputRequest::Number)),
      8 => self.io_interruption = Some(IOInterruption::Input(InputRequest::String)),
      10 => self.io_interruption = Some(IOInterruption::Halt),
      _ => {
        tracing::error!("Unhandled syscall code: {}", code);
        anyhow::bail!("Unhandled syscall code: {}", code);
      }
    }

    Ok(())
  }

  pub fn handle_input(&mut self, input: String) -> anyhow::Result<()> {
    if self.io_interruption.is_none() {
      anyhow::bail!("No input request to handle");
    }

    match self
      .io_interruption
      .take()
      .expect("No input request to handle")
    {
      IOInterruption::Input(InputRequest::Number) => {
        let value = input
          .parse::<u32>()
          .context("Could not parse input as number")?;
        self.registers[get_register_position("$v0")?] = value;
      }
      IOInterruption::Input(InputRequest::String) => {
        let address = self.registers[get_register_position("$a0")?] as usize;
        let bytes = input.as_bytes();
        self.memory[address..address + bytes.len()].copy_from_slice(bytes);
      }
      _ => anyhow::bail!("Invalid input request"),
    }

    self.io_interruption = None;
    Ok(())
  }

  pub fn search_entrypoint(
    &self,
    entrypoint: &str,
    statements: &[Statement],
  ) -> anyhow::Result<usize> {
    statements
      .iter()
      .enumerate()
      .find(|(i, statement)| {
        if let Statement::Label(label) = statement {
          if label == entrypoint {
            return true;
          }
        }

        false
      })
      .map(|(i, _)| i)
      .ok_or_else(|| anyhow::anyhow!("entrypoint not found"))
  }

  pub fn load(
    &mut self,
    Program {
      data_section,
      text_section,
    }: Program,
  ) -> anyhow::Result<()> {
    let mut data_section_offset = 0;
    for variable in data_section.variables {
      let value = variable.to_bytes();
      let size = variable.size();
      let offset = data_section_offset;
      data_section_offset += size;

      self.memory[offset..offset + size].copy_from_slice(&value);
      self.data_variables.insert(variable.name.clone(), offset);
    }

    self.pc = self
      .search_entrypoint(&text_section.entrypoint, &text_section.statements)
      .context("Could not find entrypoint")?;
    self.statements = text_section.statements;

    Ok(())
  }

  pub fn step(&mut self) -> anyhow::Result<()> {
    let statement = self.statements[self.pc].clone();
    self.pc += 1;
    if let Statement::Instruction(instruction) = statement {
      tracing::debug!(
        "[{pc}] Executing instruction: {:#?}",
        instruction,
        pc = self.pc,
      );
      self.eval_instruction(&instruction)?;
    }

    Ok(())
  }

  pub fn run(&mut self) -> anyhow::Result<()> {
    while self.pc < self.statements.len() && self.io_interruption.is_none() {
      self.step()?;

      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
  }

  pub fn get_status(&self) -> StatusUpdateMessage {
    StatusUpdateMessage {
      registers: self.registers.to_vec(),
      pc: self.pc,
      io_interruption: self.io_interruption.clone(),
      memory_patch: self.memory_patch.clone(),
    }
  }
}
