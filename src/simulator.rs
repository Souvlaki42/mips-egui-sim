use std::collections::HashMap;

use thiserror::Error;

use crate::{
    assembler::{BASE_DATA_ADDR, Instruction, MEMORY_SIZE},
    registers::{Register, RegisterError, RegisterFile},
};

#[derive(Debug, Error)]
pub enum SimulatorError {
    #[error("Register error: {0}")]
    RegisterError(#[from] RegisterError),
    #[error("No more instructions")]
    NoMoreInstructions,
}

#[derive(Debug)]
pub struct Simulator {
    memory: [u8; MEMORY_SIZE],
    registers: RegisterFile,
    instructions: HashMap<u32, Instruction>,
    pc: u32,
}

impl Simulator {
    pub fn new(instructions: HashMap<u32, Instruction>, memory: Vec<u8>, entry: u32) -> Simulator {
        let mut mem_array = [0u8; MEMORY_SIZE];
        let len = memory.len().min(MEMORY_SIZE);
        mem_array[..len].copy_from_slice(&memory[..len]);

        Simulator {
            memory: mem_array,
            registers: RegisterFile::default(),
            instructions,
            pc: entry,
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), SimulatorError> {
        match instruction {
            Instruction::AddImmediate { res, reg, imm } => {
                let value = self.registers.get(reg).wrapping_add(imm as u32);
                self.registers.set(res, value);
            }
            Instruction::LoadUpperImmediate { res, imm } => {
                let value = (imm as u32) << 16;
                self.registers.set(res, value);
            }
            Instruction::OrImmediate { res, reg, imm } => {
                let value = self.registers.get(reg) | (imm as u32);
                self.registers.set(res, value);
            }
            Instruction::SystemCall => {
                self.handle_syscall()?;
            }
        }
        Ok(())
    }

    fn handle_syscall(&mut self) -> Result<(), SimulatorError> {
        let v0 = self.registers.get(Register::V0);
        match v0 {
            4 => {
                let addr = self.registers.get(Register::A0) as usize;
                let offset = addr - BASE_DATA_ADDR as usize;

                let mut bytes = Vec::new();
                let mut i = offset;
                while i < self.memory.len() && self.memory[i] != 0 {
                    bytes.push(self.memory[i]);
                    i += 1;
                }

                let s = String::from_utf8_lossy(&bytes);
                print!("{}", s);
            }
            10 => {
                return Err(SimulatorError::NoMoreInstructions);
            }
            _ => {
                println!("Unknown syscall: {}", v0);
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), SimulatorError> {
        let instruction = self
            .instructions
            .get(&self.pc)
            .ok_or(SimulatorError::NoMoreInstructions)?
            .clone();

        self.execute_instruction(instruction)?;
        self.pc += 4;
        Ok(())
    }
}
