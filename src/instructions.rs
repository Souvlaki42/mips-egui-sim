use crate::{registers::Register, simulator::Simulator};

mod encode_format {
    use super::Register;

    pub fn r_format(opcode: u32, funct: u32, rs: &Register, rt: &Register, rd: &Register) -> u32 {
        let rs = *rs as u32;
        let rt = *rt as u32;
        let rd = *rd as u32;
        let shamt = 0; // Usually 0, except for shift instructions

        (opcode << 26) | (rs << 21) | (rt << 16) | (rd << 11) | (shamt << 6) | funct
    }

    pub fn i_format(opcode: u32, rs: &Register, rt: &Register, imm: &i16) -> u32 {
        let rs = *rs as u32;
        let rt = *rt as u32;
        let imm = *imm as u16; // Cast to u16 to preserve bit pattern

        (opcode << 26) | (rs << 21) | (rt << 16) | (imm as u32)
    }

    pub fn j_format(opcode: u32, addr: u32) -> u32 {
        (opcode << 26) | (addr & 0x3FFFFFF) // 26-bit address
    }
}

macro_rules! define_instructions {
    (
        $(
            $name:ident => $format:ident {
                opcode: $op:literal,
                $(funct: $funct:literal,)?
                $(fields: [$($field:ident: $ftype:ty),+],)?
                execute: |$sim:ident $(, $($farg:ident),+)?| $exec:block
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Instruction {
            $(
                $name $({ $($field: $ftype),+ })?,
            )*
        }

        impl Instruction {
          pub fn encode(&self) -> u32 {
              match self {
                  Self::AddUnsigned { rd, rs, rt } => {
                      encode_format::r_format(0x00, 0x21, rs, rt, rd)
                  }
                  Self::SystemCall => {
                      encode_format::r_format_no_regs(0x00, 0x0C)
                  }
                  Self::AddImmediate { rt, rs, imm } => {
                      encode_format::i_format(0x09, rs, rt, imm)
                  }
                  Self::LoadWord { rt, base, offset } => {
                      encode_format::i_format(0x23, base, rt, offset)
                  }
              }
          }

            pub fn execute(&self, sim: &mut Simulator) {
                match self {
                    $(
                        Self::$name $({ $($field),+ })? => {
                            let $sim = sim;
                            $exec
                        }
                    )*
                }
            }
        }
    };
}

define_instructions! {
    AddUnsigned => r_format {
        opcode: 0x00,
        funct: 0x21,
        fields: [rd: Register, rs: Register, rt: Register],
        execute: |s, rd, rs, rt| {
            // s.regs[*rd] = s.regs[*rs].wrapping_add(s.regs[*rt]);
        }
    },
    SystemCall => r_format {
        opcode: 0x00,
        funct: 0x0C,
        execute: |s| {
            // s.handle_syscall();
        }
    },
    AddImmediate => i_format {
        opcode: 0x09,
        fields: [rt: Register, rs: Register, imm: i16],
        execute: |s, rt, rs, imm| {
            // s.regs[*rt] = s.regs[*rs].wrapping_add(*imm as u32);
        }
    },
    LoadWord => i_format {
        opcode: 0x23,
        fields: [rt: Register, base: Register, offset: i16],
        execute: |s, rt, base, offset| {
            // let addr = s.regs[*base].wrapping_add(*offset as u32);
            // s.regs[*rt] = s.memory.load_word(addr);
        }
    },
    StoreWord => i_format {
      opcode: 0x2B,
      fields: [rt: Register, base: Register, offset: i16],
      execute: |s, rt, base, offset| {
        // let addr = s.regs[*base].wrapping_add(*offset as u32);
        // s.memory.store_word(addr, offset);
      }
    },
    LoadUpperImmediate => i_format {
      opcode: 0x0F,
      execute: |s| {}
    }
}
