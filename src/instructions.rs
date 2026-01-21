use crate::{
    registers::Register,
    simulator::{Simulator, SimulatorError},
};

// Encoding helper functions
mod encode_format {
    use super::Register;

    pub fn r_format(opcode: u32, funct: u32, rs: Register, rt: Register, rd: Register) -> u32 {
        let rs = rs as u32;
        let rt = rt as u32;
        let rd = rd as u32;
        let shamt = 0;
        (opcode << 26) | (rs << 21) | (rt << 16) | (rd << 11) | (shamt << 6) | funct
    }

    pub fn r_format_syscall(opcode: u32, funct: u32) -> u32 {
        (opcode << 26) | funct
    }

    pub fn i_format(opcode: u32, rs: Register, rt: Register, imm: i16) -> u32 {
        let rs = rs as u32;
        let rt = rt as u32;
        let imm = imm as u16;
        (opcode << 26) | (rs << 21) | (rt << 16) | (imm as u32)
    }

    pub fn j_format(opcode: u32, addr: u32) -> u32 {
        (opcode << 26) | (addr & 0x3FFFFFF)
    }
}

macro_rules! define_instructions {
    (
        $(
            $name:ident {
                format: $format:ident,
                opcode: $op:literal,
                $(funct: $funct:literal,)?
                $(fields: { $($field:ident: $ftype:ty),+ },)?
                encode: $encode_body:expr,
                execute: |$sim:ident $(, $($exec_arg:ident),+)?| $exec_body:block
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Instruction {
            $(
                $name $({
                    $($field: $ftype),+
                })?,
            )*
        }

        impl Instruction {
            pub fn encode(&self) -> u32 {
                match self {
                    $(
                        Self::$name $({ $($field),+ })? => {
                            $encode_body
                        }
                    )*
                }
            }

            pub fn execute(&self, simulator: &mut Simulator) -> Result<(), SimulatorError> {
                match self {
                    $(
                        Self::$name $({ $($field),+ })? => {
                            let $sim = simulator;
                            $($(let $exec_arg = $field;)+)?
                            $exec_body
                        }
                    )*
                }
            }
        }
    };
}

// Helper macro for decoding R-format instructions
macro_rules! decode_r_format {
    ($word:expr, $name:ident) => {
        Some(Instruction::$name)
    };
    ($word:expr, $name:ident, $f1:ident, $f2:ident, $f3:ident) => {{
        let rs = Register::try_from((($word >> 21) & 0x1F) as u8).ok()?;
        let rt = Register::try_from((($word >> 16) & 0x1F) as u8).ok()?;
        let rd = Register::try_from((($word >> 11) & 0x1F) as u8).ok()?;
        Some(Instruction::$name {
            $f1: rd,
            $f2: rs,
            $f3: rt,
        })
    }};
}

// Helper macro for decoding I-format instructions
macro_rules! decode_i_or_j_format {
    ($word:expr, i_format, $name:ident, $f1:ident, $f2:ident, $f3:ident) => {{
        let rs = Register::try_from((($word >> 21) & 0x1F) as u8).ok()?;
        let rt = Register::try_from((($word >> 16) & 0x1F) as u8).ok()?;
        let imm = ($word & 0xFFFF) as i16;
        Some(Instruction::$name {
            $f1: rt,
            $f2: rs,
            $f3: imm,
        })
    }};
}

define_instructions! {
    AddImmediate {
        format: i_format,
        opcode: 0x09,
        fields: { res: Register, reg: Register, imm: i16 },
        encode: encode_format::i_format(0x09, *reg, *res, *imm),
        execute: |s, res, reg, imm| {
            let value = s.registers.get(*reg).wrapping_add((*imm) as u32);
            s.registers.set(*res, value);
            Ok(())
        }
    },
    AddUnsigned {
        format: r_format,
        opcode: 0x00,
        funct: 0x21,
        fields: { res: Register, reg: Register, ret: Register },
        encode: encode_format::r_format(0x00, 0x21, *reg, *ret, *res),
        execute: |s, res, reg, ret| {
            let value = s.registers.get(*reg).wrapping_add(s.registers.get(*ret));
            s.registers.set(*res, value);
            Ok(())
        }
    },
    LoadUpperImmediate {
        format: i_format,
        opcode: 0x0F,
        fields: { res: Register, imm: i16 },
        encode: encode_format::i_format(0x0F, Register::Zero, *res, *imm),
        execute: |s, res, imm| {
            let value = (*imm as u32) << 16;
            s.registers.set(*res, value);
            Ok(())
        }
    },
    OrImmediate {
        format: i_format,
        opcode: 0x0D,
        fields: { res: Register, reg: Register, imm: i16 },
        encode: encode_format::i_format(0x0D, *reg, *res, *imm),
        execute: |s, res, reg, imm| {
            let value = s.registers.get(*reg) | (*imm as u32);
            s.registers.set(*res, value);
            Ok(())
        }
    },
    SystemCall {
        format: r_format,
        opcode: 0x00,
        funct: 0x0C,
        encode: encode_format::r_format_syscall(0x00, 0x0C),
        execute: |s| {
            s.handle_syscall()?;
            Ok(())
        }
    },
}
