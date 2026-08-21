use crate::types::{Op, Program, VmError};

const LOCALS_SIZE: usize = 256;

/// Executes a `Program`, writing PRINT output through the supplied
/// sink. Locals are a single flat slot array shared across calls,
/// so recursive routines must pass data on the value stack rather
/// than through LOAD/STORE (see DESIGN.md).
pub struct Vm {
    stack: Vec<i64>,
    call_stack: Vec<i64>,
    locals: [i64; LOCALS_SIZE],
    pc: i64,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            call_stack: Vec::new(),
            locals: [0; LOCALS_SIZE],
            pc: 0,
        }
    }

    pub fn run(&mut self, program: &Program, out: &mut impl std::io::Write) -> Result<(), VmError> {
        let ops = &program.ops;
        loop {
            if self.pc < 0 || self.pc as usize >= ops.len() {
                return Err(VmError::UnexpectedEnd);
            }
            let op = ops[self.pc as usize];
            let mut next_pc = self.pc + 1;

            match op {
                Op::Push(n) => self.stack.push(n),
                Op::Pop => {
                    self.pop()?;
                }
                Op::Dup => {
                    let v = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(v);
                }
                Op::Add => self.binary(|a, b| Ok(a + b))?,
                Op::Sub => self.binary(|a, b| Ok(a - b))?,
                Op::Mul => self.binary(|a, b| Ok(a * b))?,
                Op::Div => self.binary(|a, b| {
                    if b == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(a / b)
                    }
                })?,
                Op::Mod => self.binary(|a, b| {
                    if b == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(a % b)
                    }
                })?,
                Op::Neg => {
                    let a = self.pop()?;
                    self.stack.push(-a);
                }
                Op::Eq => self.binary(|a, b| Ok((a == b) as i64))?,
                Op::Lt => self.binary(|a, b| Ok((a < b) as i64))?,
                Op::Gt => self.binary(|a, b| Ok((a > b) as i64))?,
                Op::Load(idx) => {
                    let slot = self.local_index(idx)?;
                    self.stack.push(self.locals[slot]);
                }
                Op::Store(idx) => {
                    let slot = self.local_index(idx)?;
                    let v = self.pop()?;
                    self.locals[slot] = v;
                }
                Op::Jmp(target) => next_pc = target,
                Op::Jz(target) => {
                    let v = self.pop()?;
                    if v == 0 {
                        next_pc = target;
                    }
                }
                Op::Jnz(target) => {
                    let v = self.pop()?;
                    if v != 0 {
                        next_pc = target;
                    }
                }
                Op::Call(target) => {
                    self.call_stack.push(next_pc);
                    next_pc = target;
                }
                Op::Ret => {
                    next_pc = self.call_stack.pop().ok_or(VmError::CallStackUnderflow)?;
                }
                Op::Print => {
                    let v = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                    writeln!(out, "{}", v).map_err(|_| VmError::UnexpectedEnd)?;
                }
                Op::Halt => return Ok(()),
            }

            if next_pc < 0 || next_pc as usize > ops.len() {
                return Err(VmError::JumpOutOfBounds(next_pc));
            }
            self.pc = next_pc;
        }
    }

    fn pop(&mut self) -> Result<i64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn binary(&mut self, f: impl Fn(i64, i64) -> Result<i64, VmError>) -> Result<(), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(f(a, b)?);
        Ok(())
    }

    fn local_index(&self, idx: i64) -> Result<usize, VmError> {
        if idx < 0 || idx as usize >= LOCALS_SIZE {
            return Err(VmError::LocalOutOfBounds(idx));
        }
        Ok(idx as usize)
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs a program and returns everything written to PRINT, one line
/// per call, as a convenience for tests and the CLI.
pub fn run_collect(program: &Program) -> Result<String, VmError> {
    let mut out = Vec::new();
    let mut vm = Vm::new();
    vm.run(program, &mut out)?;
    Ok(String::from_utf8_lossy(&out).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;

    fn run_src(src: &str) -> Result<String, VmError> {
        let prog = assemble(src).expect("assembles");
        run_collect(&prog)
    }

    #[test]
    fn arithmetic_and_print() {
        let out = run_src("PUSH 2\nPUSH 3\nADD\nPRINT\nHALT\n").unwrap();
        assert_eq!(out, "5\n");
    }

    #[test]
    fn stack_underflow_on_pop() {
        let err = run_src("POP\nHALT\n").unwrap_err();
        assert_eq!(err, VmError::StackUnderflow);
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let err = run_src("PUSH 1\nPUSH 0\nDIV\nHALT\n").unwrap_err();
        assert_eq!(err, VmError::DivisionByZero);
    }

    #[test]
    fn locals_round_trip() {
        let out = run_src("PUSH 42\nSTORE 0\nLOAD 0\nPRINT\nHALT\n").unwrap();
        assert_eq!(out, "42\n");
    }

    #[test]
    fn undefined_local_out_of_bounds() {
        let err = run_src("PUSH 1\nSTORE 9999\nHALT\n").unwrap_err();
        assert_eq!(err, VmError::LocalOutOfBounds(9999));
    }
}
