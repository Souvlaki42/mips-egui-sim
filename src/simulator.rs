use crate::{
    assembler::Assembler,
    cpu::{Cpu, CpuError, Register},
    tokenizer::tokenize,
};

#[derive(Debug)]
pub struct Simulator {
    memory: Vec<u8>,
    cpu: Cpu,
    assembler: Assembler,
}

const MEMORY_SIZE: usize = 1024 * 1024;

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            memory: vec![0; MEMORY_SIZE],
            cpu: Cpu::new(),
            assembler: Assembler::new(),
        }
    }

    fn get_register(&self, register: &str) -> Result<u32, CpuError> {
        let reg = register.parse::<Register>()?;
        Ok(self.cpu.gprs.get(reg))
    }

    fn set_register(&mut self, register: &str, value: u32) -> Result<(), CpuError> {
        let reg = register.parse::<Register>()?;
        self.cpu.gprs.set(reg, value);
        Ok(())
    }

    pub fn run(&mut self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tokens = tokenize(file)?;

        self.assembler.assemble(tokens)?;

        println!("{:?}", self.assembler);

        Ok(())
    }
}
