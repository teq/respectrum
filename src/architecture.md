Modules:

* Memory
  - Implements `Index<u16>` (for byte read & write)
* Decoder - opcode parser
  - Wraps memory
  - Implements `read_token` which advances in memory and returns next acknowledged CPU token
  - Affected by previously parsed prefixes
* Executor - opcode executor
  - Wraps decoder
  - Implements `exec_token` which reads tokens from decoder and executes them on emulated CPU
* Disassembler
  - Wraps decoder
  - Implements `format_token`

Execution: executor(decoder(memory()))

Disassembly: disassembler(decoder(memory()))
