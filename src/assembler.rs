use crate::types::{opcode, AsmError, Op, Program};
use std::collections::HashMap;

/// Strips a `;` comment and surrounding whitespace from one line.
fn strip_comment(line: &str) -> &str {
    match line.find(';') {
        Some(idx) => line[..idx].trim(),
        None => line.trim(),
    }
}

/// Assembles source text into a `Program`. Two passes: the first walks
/// every non-blank line, records label addresses (a label does not
/// occupy a slot, it names the address of the next instruction), the
/// second parses each instruction and resolves label operands.
pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let mut labels: HashMap<String, i64> = HashMap::new();
    let mut instr_lines: Vec<(usize, &str)> = Vec::new();

    for (i, raw) in source.lines().enumerate() {
        let line = strip_comment(raw);
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_suffix(':') {
            let name = name.trim();
            if labels.contains_key(name) {
                return Err(AsmError::DuplicateLabel {
                    line: i + 1,
                    label: name.to_string(),
                });
            }
            labels.insert(name.to_string(), instr_lines.len() as i64);
            continue;
        }
        instr_lines.push((i + 1, line));
    }

    let mut ops = Vec::with_capacity(instr_lines.len());
    for (lineno, text) in instr_lines {
        ops.push(parse_instruction(lineno, text, &labels)?);
    }

    Ok(Program { ops })
}

fn parse_instruction(
    lineno: usize,
    text: &str,
    labels: &HashMap<String, i64>,
) -> Result<Op, AsmError> {
    let mut parts = text.split_whitespace();
    let mnemonic = parts.next().unwrap_or("");
    let rest: Vec<&str> = parts.collect();

    let want_int = |rest: &[&str]| -> Result<i64, AsmError> {
        let tok = rest.first().ok_or_else(|| AsmError::MissingOperand {
            line: lineno,
            text: text.to_string(),
        })?;
        tok.parse::<i64>().map_err(|_| AsmError::BadOperand {
            line: lineno,
            text: text.to_string(),
        })
    };

    let want_label = |rest: &[&str]| -> Result<i64, AsmError> {
        let tok = rest.first().ok_or_else(|| AsmError::MissingOperand {
            line: lineno,
            text: text.to_string(),
        })?;
        labels
            .get(*tok)
            .copied()
            .ok_or_else(|| AsmError::UndefinedLabel {
                label: tok.to_string(),
            })
    };

    let op = match mnemonic.to_ascii_uppercase().as_str() {
        "PUSH" => Op::Push(want_int(&rest)?),
        "POP" => Op::Pop,
        "DUP" => Op::Dup,
        "ADD" => Op::Add,
        "SUB" => Op::Sub,
        "MUL" => Op::Mul,
        "DIV" => Op::Div,
        "MOD" => Op::Mod,
        "NEG" => Op::Neg,
        "EQ" => Op::Eq,
        "LT" => Op::Lt,
        "GT" => Op::Gt,
        "LOAD" => Op::Load(want_int(&rest)?),
        "STORE" => Op::Store(want_int(&rest)?),
        "JMP" => Op::Jmp(want_label(&rest)?),
        "JZ" => Op::Jz(want_label(&rest)?),
        "JNZ" => Op::Jnz(want_label(&rest)?),
        "CALL" => Op::Call(want_label(&rest)?),
        "RET" => Op::Ret,
        "PRINT" => Op::Print,
        "HALT" => Op::Halt,
        _ => {
            return Err(AsmError::UnknownInstruction {
                line: lineno,
                text: text.to_string(),
            })
        }
    };
    Ok(op)
}

/// Encodes a program to the `.mbc` binary format: a 4 byte magic
/// header, a 4 byte little-endian instruction count, then each
/// instruction as 1 opcode byte followed by 8 bytes of operand
/// (zero when the instruction takes none).
const MAGIC: &[u8; 4] = b"MRG1";

pub fn encode(program: &Program) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 4 + program.ops.len() * 9);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(program.ops.len() as u32).to_le_bytes());
    for op in &program.ops {
        out.push(op.code());
        out.extend_from_slice(&op.operand().unwrap_or(0).to_le_bytes());
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    TooShort,
    BadMagic,
    Truncated,
    UnknownOpcode(u8),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::TooShort => write!(f, "file too short to be mirage bytecode"),
            DecodeError::BadMagic => write!(f, "bad magic header, not a mirage .mbc file"),
            DecodeError::Truncated => write!(f, "truncated instruction stream"),
            DecodeError::UnknownOpcode(b) => write!(f, "unknown opcode 0x{:02x}", b),
        }
    }
}

impl std::error::Error for DecodeError {}

pub fn decode(bytes: &[u8]) -> Result<Program, DecodeError> {
    if bytes.len() < 8 {
        return Err(DecodeError::TooShort);
    }
    if &bytes[0..4] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let mut ops = Vec::with_capacity(count);
    let mut pos = 8;
    for _ in 0..count {
        if pos + 9 > bytes.len() {
            return Err(DecodeError::Truncated);
        }
        let code = bytes[pos];
        let operand = i64::from_le_bytes(bytes[pos + 1..pos + 9].try_into().unwrap());
        pos += 9;
        let op = match code {
            opcode::PUSH => Op::Push(operand),
            opcode::POP => Op::Pop,
            opcode::DUP => Op::Dup,
            opcode::ADD => Op::Add,
            opcode::SUB => Op::Sub,
            opcode::MUL => Op::Mul,
            opcode::DIV => Op::Div,
            opcode::MOD => Op::Mod,
            opcode::NEG => Op::Neg,
            opcode::EQ => Op::Eq,
            opcode::LT => Op::Lt,
            opcode::GT => Op::Gt,
            opcode::LOAD => Op::Load(operand),
            opcode::STORE => Op::Store(operand),
            opcode::JMP => Op::Jmp(operand),
            opcode::JZ => Op::Jz(operand),
            opcode::JNZ => Op::Jnz(operand),
            opcode::CALL => Op::Call(operand),
            opcode::RET => Op::Ret,
            opcode::PRINT => Op::Print,
            opcode::HALT => Op::Halt,
            other => return Err(DecodeError::UnknownOpcode(other)),
        };
        ops.push(op);
    }
    Ok(Program { ops })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_and_blank_lines() {
        let src = "; a comment\nPUSH 1 ; push one\n\nHALT\n";
        let prog = assemble(src).unwrap();
        assert_eq!(prog.ops, vec![Op::Push(1), Op::Halt]);
    }

    #[test]
    fn resolves_forward_and_backward_labels() {
        let src = "JMP skip\nPUSH 99\nskip:\nPUSH 1\nloop:\nJMP loop\n";
        let prog = assemble(src).unwrap();
        assert_eq!(
            prog.ops,
            vec![Op::Jmp(2), Op::Push(99), Op::Push(1), Op::Jmp(3)]
        );
    }

    #[test]
    fn undefined_label_is_an_error() {
        let err = assemble("JMP nowhere\n").unwrap_err();
        assert_eq!(
            err,
            AsmError::UndefinedLabel {
                label: "nowhere".to_string()
            }
        );
    }

    #[test]
    fn duplicate_label_is_an_error() {
        let err = assemble("a:\nPUSH 1\na:\nHALT\n").unwrap_err();
        assert!(matches!(err, AsmError::DuplicateLabel { .. }));
    }

    #[test]
    fn round_trips_through_bytecode() {
        let prog = assemble("PUSH 5\nPUSH 3\nADD\nPRINT\nHALT\n").unwrap();
        let bytes = encode(&prog);
        let back = decode(&bytes).unwrap();
        assert_eq!(prog.ops, back.ops);
    }
}
