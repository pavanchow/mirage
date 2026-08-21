use std::fmt;

/// A single decoded instruction. Labels are resolved to instruction
/// indices by the assembler before this type is ever constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Push(i64),
    Pop,
    Dup,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Eq,
    Lt,
    Gt,
    Load(i64),
    Store(i64),
    Jmp(i64),
    Jz(i64),
    Jnz(i64),
    Call(i64),
    Ret,
    Print,
    Halt,
}

/// Opcode bytes, shared by the binary encoder and decoder.
pub mod opcode {
    pub const PUSH: u8 = 0;
    pub const POP: u8 = 1;
    pub const DUP: u8 = 2;
    pub const ADD: u8 = 3;
    pub const SUB: u8 = 4;
    pub const MUL: u8 = 5;
    pub const DIV: u8 = 6;
    pub const MOD: u8 = 7;
    pub const NEG: u8 = 8;
    pub const EQ: u8 = 9;
    pub const LT: u8 = 10;
    pub const GT: u8 = 11;
    pub const LOAD: u8 = 12;
    pub const STORE: u8 = 13;
    pub const JMP: u8 = 14;
    pub const JZ: u8 = 15;
    pub const JNZ: u8 = 16;
    pub const CALL: u8 = 17;
    pub const RET: u8 = 18;
    pub const PRINT: u8 = 19;
    pub const HALT: u8 = 20;
}

impl Op {
    pub fn code(&self) -> u8 {
        use opcode::*;
        match self {
            Op::Push(_) => PUSH,
            Op::Pop => POP,
            Op::Dup => DUP,
            Op::Add => ADD,
            Op::Sub => SUB,
            Op::Mul => MUL,
            Op::Div => DIV,
            Op::Mod => MOD,
            Op::Neg => NEG,
            Op::Eq => EQ,
            Op::Lt => LT,
            Op::Gt => GT,
            Op::Load(_) => LOAD,
            Op::Store(_) => STORE,
            Op::Jmp(_) => JMP,
            Op::Jz(_) => JZ,
            Op::Jnz(_) => JNZ,
            Op::Call(_) => CALL,
            Op::Ret => RET,
            Op::Print => PRINT,
            Op::Halt => HALT,
        }
    }

    pub fn operand(&self) -> Option<i64> {
        match self {
            Op::Push(n) | Op::Load(n) | Op::Store(n) | Op::Jmp(n) | Op::Jz(n) | Op::Jnz(n)
            | Op::Call(n) => Some(*n),
            _ => None,
        }
    }

    pub fn mnemonic(&self) -> &'static str {
        match self {
            Op::Push(_) => "PUSH",
            Op::Pop => "POP",
            Op::Dup => "DUP",
            Op::Add => "ADD",
            Op::Sub => "SUB",
            Op::Mul => "MUL",
            Op::Div => "DIV",
            Op::Mod => "MOD",
            Op::Neg => "NEG",
            Op::Eq => "EQ",
            Op::Lt => "LT",
            Op::Gt => "GT",
            Op::Load(_) => "LOAD",
            Op::Store(_) => "STORE",
            Op::Jmp(_) => "JMP",
            Op::Jz(_) => "JZ",
            Op::Jnz(_) => "JNZ",
            Op::Call(_) => "CALL",
            Op::Ret => "RET",
            Op::Print => "PRINT",
            Op::Halt => "HALT",
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.operand() {
            Some(n) => write!(f, "{} {}", self.mnemonic(), n),
            None => write!(f, "{}", self.mnemonic()),
        }
    }
}

/// A fully assembled program: instructions plus the label table used
/// to render addresses back to names when disassembling.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmError {
    UnknownInstruction { line: usize, text: String },
    BadOperand { line: usize, text: String },
    MissingOperand { line: usize, text: String },
    DuplicateLabel { line: usize, label: String },
    UndefinedLabel { label: String },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmError::UnknownInstruction { line, text } => {
                write!(f, "line {}: unknown instruction '{}'", line, text)
            }
            AsmError::BadOperand { line, text } => {
                write!(f, "line {}: bad operand in '{}'", line, text)
            }
            AsmError::MissingOperand { line, text } => {
                write!(f, "line {}: missing operand in '{}'", line, text)
            }
            AsmError::DuplicateLabel { line, label } => {
                write!(f, "line {}: duplicate label '{}'", line, label)
            }
            AsmError::UndefinedLabel { label } => {
                write!(f, "undefined label '{}'", label)
            }
        }
    }
}

impl std::error::Error for AsmError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    StackUnderflow,
    CallStackUnderflow,
    DivisionByZero,
    UnknownOpcode(u8),
    JumpOutOfBounds(i64),
    LocalOutOfBounds(i64),
    UnexpectedEnd,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow => write!(f, "stack underflow"),
            VmError::CallStackUnderflow => write!(f, "call stack underflow (RET with no CALL)"),
            VmError::DivisionByZero => write!(f, "division by zero"),
            VmError::UnknownOpcode(b) => write!(f, "unknown opcode 0x{:02x}", b),
            VmError::JumpOutOfBounds(addr) => write!(f, "jump target out of bounds: {}", addr),
            VmError::LocalOutOfBounds(idx) => write!(f, "local index out of bounds: {}", idx),
            VmError::UnexpectedEnd => write!(f, "program ended without HALT"),
        }
    }
}

impl std::error::Error for VmError {}
