use celestial_hub_astrolabe::ast::{
  Instruction, InstructionArgument, Program, Statement, TextSection,
};

use crate::vm::eval::utils::{memory_map::Memory, register::RegisterPosition};

use self::utils::register::get_register_position;

use super::VM;
pub mod utils;

impl VM {
  pub fn eval(
    &mut self,
    Program {
      data_section,
      text_section,
    }: Program,
  ) -> anyhow::Result<()> {
    // Allocate memory for data section
    let mut data_section_offset = 0;
    for variable in data_section.variables {
      let value = variable.to_bytes();
      let size = variable.size();
      let offset = data_section_offset;
      data_section_offset += size;

      self.memory[offset..offset + size].copy_from_slice(&value);
      self.data_variables.insert(variable.name.clone(), offset);

      println!(
        "{name}: {data:?}",
        name = &variable.name,
        data = &self.memory[offset..offset + size]
      );
    }

    let TextSection {
      entrypoint,
      statements,
    } = text_section;

    let entrypoint = self.search_entrypoint(&entrypoint, &statements)?;

    self.pc = entrypoint;

    while self.pc < statements.len() {
      let statement = &statements[self.pc];
      self.pc += 1;

      match statement {
        Statement::Label(_) => {}
        Statement::Instruction(instruction) => {
          self.eval_instruction(instruction)?;
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
      Instruction::Syscall => {
        let syscall_code = self.registers[get_register_position("$v0")?];

        // TODO: Delegate the IO to a subscriber that will handle it
        match syscall_code {
          // Print integer
          1 => {
            let value = self.registers[get_register_position("$a0")?];

            println!("{}", value);
          }
          // Print string
          4 => {
            let address = self.registers[get_register_position("$a0")?];

            let mut i = address as usize;
            while self.memory[i] != 0 {
              print!("{}", self.memory[i] as char);
              i += 1;
            }
          }
          // Read integer
          5 => {
            let mut value = String::new();
            std::io::stdin().read_line(&mut value)?;

            let value = value.trim().parse::<u32>()?;

            self.registers[get_register_position("$v0")?] = value;
          }
          // Read string
          8 => {
            let address = self.registers[get_register_position("$a0")?];
            let length = self.registers[get_register_position("$a1")?];

            let i = address as usize;
            let mut value = String::new();
            std::io::stdin().read_line(&mut value)?;

            let value = value.trim().as_bytes();

            self.memory[i..i + length as usize].copy_from_slice(value);
          }
          // Exit
          10 => {
            std::process::exit(0);
          }
          _ => unreachable!(),
        }

        Ok(())
      }
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

  pub fn search_entrypoint(
    &self,
    entrypoint: &str,
    statements: &[Statement],
  ) -> anyhow::Result<usize> {
    statements
      .iter()
      .enumerate()
      .find(|(_, statement)| {
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
}
