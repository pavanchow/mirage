# Mirage design

## Instruction set

Twenty instructions, listed in `src/types.rs` as the `Op` enum. Seven of them carry an `i64` operand: `PUSH` (a literal), `LOAD`/`STORE` (a local slot index), and `JMP`/`JZ`/`JNZ`/`CALL` (an instruction address, resolved from a label at assemble time). The rest take no operand.

Arithmetic and comparisons pop their operands from the value stack and push exactly one result. `EQ`, `LT`, and `GT` push `1` for true and `0` for false, matching `JZ`/`JNZ` which treat `0` as false and anything else as true. `DIV` and `MOD` check the divisor and return `VmError::DivisionByZero` rather than trapping.

## Bytecode format

The `.mbc` file is a flat binary encoding, defined in `src/assembler.rs`:

```
4 bytes   magic "MRG1"
4 bytes   instruction count, little-endian u32
for each instruction:
  1 byte    opcode
  8 bytes   operand, little-endian i64 (zero when the instruction has none)
```

Every instruction is a fixed 9 bytes after the header, so the format has no variable-length encoding to get wrong and disassembly can index straight into the stream. Label names are not preserved: the assembler resolves every label to an instruction index during assembly, and disassembly prints those indices as numeric jump targets rather than reconstructed names.

## Assembler pipeline

`assemble()` in `src/assembler.rs` runs two passes over the source text:

1. **Label pass.** Walk every line, strip comments and whitespace. A line ending in `:` defines a label at the address equal to the number of real instructions seen so far (labels do not occupy a slot). Every other non-blank line is queued as an instruction to parse.
2. **Instruction pass.** Parse each queued line into an `Op`, looking up any label operand in the table built during the first pass. An operand that is not a known label is `AsmError::UndefinedLabel`. Because labels are collected before any instruction is parsed, both forward and backward references resolve correctly.

Parse errors report the source line number and the offending text: unknown mnemonic, a non-integer operand where one is required, a missing operand, or a label defined twice.

## Executor pipeline

`Vm::run()` in `src/vm.rs` is a straightforward fetch-decode-execute loop over a program counter, a `Vec<i64>` value stack, a `Vec<i64>` call stack of return addresses, and a fixed-size local slot array. `CALL` pushes the address of the following instruction onto the call stack and jumps. `RET` pops that address and jumps back. Locals are one flat array shared by the whole run rather than one array per call frame, so a recursive routine has to pass its arguments and results through the value stack instead of through `LOAD`/`STORE`. `examples/factorial.asm` is written that way on purpose, as a working demonstration of the constraint.

## Error handling

Every failure mode is a variant of `VmError` or `AsmError`, both plain enums that implement `std::error::Error`. There is no `panic!`, `unwrap()`, or array indexing without a bounds check anywhere on the execution path: stack access goes through `pop()` which turns an empty stack into `VmError::StackUnderflow`, local access is range-checked into `VmError::LocalOutOfBounds`, jump targets are range-checked after every instruction into `VmError::JumpOutOfBounds`, and a stream that runs off the end without hitting `HALT` returns `VmError::UnexpectedEnd`. Malformed bytecode read from disk gets its own `DecodeError` (short file, bad magic, truncated stream, unknown opcode byte) so a corrupt `.mbc` file is also a typed error rather than a crash.
