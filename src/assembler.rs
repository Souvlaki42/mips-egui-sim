use std::{collections::HashMap, ffi::CString, iter::Peekable, slice::Iter, str::FromStr};

use thiserror::Error;

use crate::{
    cpu::Register,
    tokenizer::{Directive, Token},
};

pub const BASE_TEXT_ADDR: usize = 0x0040_0000;
pub const BASE_DATA_ADDR: usize = 0x1001_0000;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Segment {
    Text,
    Data,
}

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Unknown directive")]
    UnknownDirective,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Entrypoint missing")]
    EntrypointMissing,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid register")]
    InvalidRegister,
    #[error("Invalid label")]
    InvalidLabel,
    #[error("Invalid string")]
    InvalidString,
}

pub struct Symbol {
    address: usize,
    segment: Segment,
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Symbol")
            .field("address", &format_args!("0x{:08X}", self.address))
            .field("segment", &self.segment)
            .finish()
    }
}

pub struct Assembler {
    symbols: HashMap<String, Symbol>,
    data_addr: usize, // Starts 0x10010000
    text_addr: usize, // Starts 0x00400000
    entry_point: Option<String>,
    memory: Vec<u8>, // Unified memory
    text_lines: Vec<Instruction>,
    current_segment: Segment,
}

impl std::fmt::Debug for Assembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assembler")
            .field("symbols", &self.symbols)
            .field("data_addr", &format_args!("0x{:08X}", self.data_addr))
            .field("text_addr", &format_args!("0x{:08X}", self.text_addr))
            .field("entry_point", &self.entry_point)
            .field("text_lines", &self.text_lines)
            .finish()
    }
}

#[derive(Debug)]
pub enum Instruction {
    AddImmediate {
        res: Register,
        reg: Register,
        imm: i32,
    },
    LoadUpperImmediate {
        res: Register,
        imm: i32,
    },
    OrImmediate {
        res: Register,
        reg: Register,
        imm: i32,
    },
    SystemCall,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            data_addr: BASE_DATA_ADDR,
            text_addr: BASE_TEXT_ADDR,
            entry_point: None,
            memory: Vec::new(),
            text_lines: Vec::new(),
            current_segment: Segment::Text,
        }
    }

    pub fn assemble(&mut self, tokenized: Vec<Vec<Token>>) -> Result<(), AssemblerError> {
        for line_tokens in tokenized {
            let mut tokens = line_tokens.iter().peekable();

            if let Some(Token::Label { name, decl: true }) = tokens.peek() {
                let addr = match self.current_segment {
                    Segment::Data => self.data_addr,
                    Segment::Text => self.text_addr,
                };
                self.symbols.insert(
                    name.clone(),
                    Symbol {
                        address: addr,
                        segment: self.current_segment,
                    },
                );
                tokens.next();
            }

            match tokens.next() {
                Some(Token::Directive { kind }) => self.handle_directive(kind, &mut tokens)?,
                Some(token) if matches!(token, Token::Operator { .. }) => {
                    let expanded = self.expand_instruction(line_tokens)?;
                    self.text_lines.extend(expanded);
                }
                None => continue,
                _ => return Err(AssemblerError::InvalidToken),
            }
        }

        Ok(())
    }

    pub fn expand_instruction(
        &mut self,
        tokens: Vec<Token>,
    ) -> Result<Vec<Instruction>, AssemblerError> {
        let mut iter = tokens.iter().peekable();
        if let Some(Token::Operator { value }) = iter.next() {
            let value_str = value.as_str();
            match value_str {
                "syscall" => return Ok(vec![Instruction::SystemCall]),
                "li" => {
                    let res = match iter.next() {
                        Some(Token::Register { value }) => value
                            .parse::<Register>()
                            .map_err(|_| AssemblerError::InvalidRegister)?,
                        _ => return Err(AssemblerError::InvalidInstruction),
                    };
                    let imm = match iter.next() {
                        Some(Token::Decimal { value }) => value,
                        _ => return Err(AssemblerError::InvalidInstruction),
                    };
                    return Ok(vec![Instruction::AddImmediate {
                        res,
                        reg: Register::ZERO,
                        imm: *imm,
                    }]);
                }
                "la" => {
                    let res = match iter.next() {
                        Some(Token::Register { value }) => value
                            .parse::<Register>()
                            .map_err(|_| AssemblerError::InvalidRegister)?,
                        _ => return Err(AssemblerError::InvalidInstruction),
                    };
                    let label = match iter.next() {
                        Some(Token::Label { name, decl: false }) => name,
                        _ => return Err(AssemblerError::InvalidInstruction),
                    };
                    let address = self
                        .symbols
                        .get(label)
                        .ok_or(AssemblerError::InvalidLabel)?
                        .address;

                    let high = address >> 16;
                    let low = address & 0xffff;

                    return Ok(vec![
                        Instruction::LoadUpperImmediate {
                            res,
                            imm: high as i32,
                        },
                        Instruction::OrImmediate {
                            res,
                            reg: res,
                            imm: low as i32,
                        },
                    ]);
                }
                _ => {}
            }
        }
        Err(AssemblerError::InvalidInstruction)
    }

    pub fn get_entry_point(&self) -> Option<String> {
        let entry = self.entry_point.clone()?;
        let entry_symbol = self.symbols.get(&entry)?;
        Some(entry_symbol.address.to_string())
    }

    pub fn memory_at(&self, addr: usize) -> &[u8] {
        let offset = if addr >= BASE_DATA_ADDR {
            addr - BASE_DATA_ADDR
        } else if addr >= BASE_TEXT_ADDR {
            addr - BASE_TEXT_ADDR
        } else {
            return &[];
        };
        self.memory.get(offset..).unwrap_or(&[])
    }

    fn handle_directive(
        &mut self,
        kind: &Directive,
        tokens: &mut Peekable<Iter<Token>>,
    ) -> Result<(), AssemblerError> {
        match kind {
            Directive::DataDirective => {
                self.current_segment = Segment::Data;
                Ok(())
            }
            Directive::TextDirective => {
                self.current_segment = Segment::Text;
                Ok(())
            }
            Directive::GlobalDirective => {
                if let Some(Token::Label { name, decl: false }) = tokens.next() {
                    self.entry_point = Some(name.clone());
                    Ok(())
                } else {
                    Err(AssemblerError::EntrypointMissing)
                }
            }
            Directive::AsciizDirective => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(&value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes_with_nul();
                    let start_offset = self.data_addr - BASE_DATA_ADDR;
                    let end_offset = start_offset + bytes.len();
                    self.memory
                        .resize(std::cmp::max(self.memory.len(), end_offset), 0);
                    self.memory[start_offset..end_offset].copy_from_slice(&bytes);
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            Directive::AsciiDirective => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(&value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes();
                    let start_offset = self.data_addr - BASE_DATA_ADDR;
                    let end_offset = start_offset + bytes.len();
                    self.memory
                        .resize(std::cmp::max(self.memory.len(), end_offset), 0);
                    self.memory[start_offset..end_offset].copy_from_slice(&bytes);
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            _ => Err(AssemblerError::UnknownDirective),
        }
    }
}
