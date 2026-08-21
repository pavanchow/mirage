# Mirage

**A small stack-based bytecode virtual machine in Rust, with its own instruction set, a text assembler, and a runner.**

Mirage is a from-scratch VM built to be read in one sitting. It has a value stack, a call stack, a flat local variable slot array, and a compact binary bytecode format. Programs are written in a plain text assembly, assembled into `.mbc` bytecode, and executed directly or disassembled back to readable form. Bad input never crashes the VM. Stack underflow, division by zero, an unknown opcode, and a jump to an undefined label all come back as typed errors.

## Instruction set

| Instruction | Effect |
|---|---|
| `PUSH n` | push integer literal `n` |
| `POP` | discard the top value |
| `DUP` | duplicate the top value |
| `ADD` `SUB` `MUL` `DIV` `MOD` | pop two, push the result, `DIV`/`MOD` by zero is an error |
| `NEG` | pop one, push its negation |
| `EQ` `LT` `GT` | pop two, push `1` or `0` |
| `LOAD n` | push the value in local slot `n` |
| `STORE n` | pop the top value into local slot `n` |
| `JMP label` | jump unconditionally |
| `JZ label` | pop, jump if the value is zero |
| `JNZ label` | pop, jump if the value is nonzero |
| `CALL label` | push a return address, jump |
| `RET` | pop a return address, jump back |
| `PRINT` | print the top value without popping it |
| `HALT` | stop execution |

Labels look like `loop:` on their own line. `;` starts a comment that runs to end of line.

## Sample program

```asm
; sum 1..=10, prints 55
    PUSH 0
    STORE 0
    PUSH 1
    STORE 1
loop:
    LOAD 1
    PUSH 10
    GT
    JNZ done
    LOAD 0
    LOAD 1
    ADD
    STORE 0
    LOAD 1
    PUSH 1
    ADD
    STORE 1
    JMP loop
done:
    LOAD 0
    PRINT
    HALT
```

## Usage

```sh
# assemble and run a program in one step
mirage run examples/sum_loop.asm

# assemble to a bytecode file
mirage asm examples/factorial.asm -o factorial.mbc

# disassemble a bytecode file back to readable instructions
mirage disasm factorial.mbc
```

## Building and testing

```sh
cargo build
cargo test
```

Tests assemble and run `examples/factorial.asm`, `examples/fibonacci.asm`, and `examples/sum_loop.asm`, then assert on the exact printed output. Separate tests check that stack underflow, division by zero, and undefined labels return errors instead of panicking.

See `DESIGN.md` for the bytecode format and the assembler and executor pipeline.

By Pavan Nallamothu.
