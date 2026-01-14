use std::{
    collections::HashMap,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    assembler::{BASE_DATA_ADDR, Instruction, MEMORY_SIZE},
    registers::{Register, RegisterError, RegisterFile},
};

#[derive(Debug, Error)]
pub enum SimulatorError {
    #[error("Register error: {0}")]
    RegisterError(#[from] RegisterError),
    #[error("Unknown syscall: {0}")]
    UnknownSyscall(u32),
    #[error("No more instructions: {0}")]
    NoMoreInstructions(u32),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Wrong input type: {0}")]
    WrongInputType(String),
    #[error("Invalid system time: {0}")]
    InvalidSystemTime(#[from] SystemTimeError),
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

    fn get_user_input(&mut self) -> Result<String, SimulatorError> {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| SimulatorError::IoError(e))?;
        Ok(input)
    }

    fn handle_syscall(&mut self) -> Result<(), SimulatorError> {
        let v0 = self.registers.get(Register::V0);
        match v0 {
            1 => {
                let value = self.registers.get(Register::A0);
                print!("{}", value);
            }
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
            5 => {
                let input = self.get_user_input()?;
                let value = input
                    .parse::<u32>()
                    .map_err(|_| SimulatorError::WrongInputType(input))?;
                self.registers.set(Register::V0, value);
            }
            10 => {
                return Err(SimulatorError::NoMoreInstructions(0));
            }
            17 => {
                let value = self.registers.get(Register::A0);
                return Err(SimulatorError::NoMoreInstructions(value));
            }
            30 => {
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| SimulatorError::InvalidSystemTime(e))?;

                let millis = duration.as_millis() as u64;

                let low = (millis & 0xFFFFFFFF) as u32;
                let high = (millis >> 32) as u32;

                self.registers.set(Register::A0, low);
                self.registers.set(Register::A1, high);
            }
            _ => {
                return Err(SimulatorError::UnknownSyscall(v0));
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), SimulatorError> {
        let instruction = self
            .instructions
            .get(&self.pc)
            .ok_or(SimulatorError::NoMoreInstructions(0))?
            .clone();

        self.execute_instruction(instruction)?;
        self.pc += 4;
        Ok(())
    }
}
