use std::{collections::HashMap, ffi::CString, iter::Peekable, slice::Iter, str::FromStr};

use thiserror::Error;

use crate::sim::tokenizer::{Directive, Token};

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
    #[error("Invalid string")]
    InvalidString,
}

#[derive(Debug)]
pub struct Symbol {
    address: usize,
    segment: Segment,
}

#[derive(Debug)]
pub struct Assembler {
    symbols: HashMap<String, Symbol>,
    data_addr: usize, // Starts 0x10010000
    text_addr: usize, // Starts 0x00400000
    entry_point: Option<String>,
    memory: Vec<u8>, // Unified memory
    text_lines: Vec<Instruction>,
    current_segment: Segment,
}

#[derive(Debug)]
pub enum Instruction {
    Pending(Vec<Token>),
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
                    self.text_lines.push(Instruction::Pending(line_tokens))
                }
                None => continue,
                _ => return Err(AssemblerError::InvalidToken),
            }
        }

        Ok(())
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
