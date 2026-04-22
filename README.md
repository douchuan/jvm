# jvm

A JVM implementation written in Rust with an LLVM-based JIT compiler.

## Features

- **Class loading** — directory, JAR, and JImage (JDK 9+ `lib/modules`) sources
- **Interpreter** — all 202 JVM opcodes implemented (one file per opcode group)
- **LLVM JIT** — ~110 opcodes translated to LLVM IR via inkwell (int/long/float/double arithmetic, bitwise, stack ops, type conversions, branching). Falls back to interpreter for uncompiled methods
- **Object model** — slot-based heap (`Oop::Ref(u32)`), zero `unsafe` for object access. Safe `Monitor` via `std::sync::{Mutex, Condvar}`
- **JNI** — ~30 native method implementations covering `java.lang.*`, `java.io.*`, `sun.misc.*`, `sun.reflect.*`
- **Threading** — Java threads mapped to OS threads, with pool management

## Workspace Structure

```
jvm/                     # Binary crate — CLI entry point
crates/
  classfile/             # JVM class file format type definitions (no dependencies)
  class-parser/          # Bytes → ClassFile parser (Cursor + Read)
  vm/                    # Core VM: interpreter, JIT, oop model, native methods
  class-verification/    # Class verification (skeleton)
tools/
  javap/                 # Class file disassembler (javap-style output)
```

### Dependency Graph

```
jvm (binary) → vm → class-parser → classfile
                          ↘ classfile
javap (tool) → class-parser → classfile
                  ↘ classfile
```

All dependencies are centralized in the root `Cargo.toml` under `[workspace.dependencies]`.
Sub-crates reference them with `dep.workspace = true`.

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run -p jvm -- --classpath /path/to/classes MyMainClass
```

See `scripts/dev.sh` for convenience commands.

## Architecture

### classfile

Pure type definitions matching the JVM class file specification. No parsing logic.

### class-parser

Reads raw bytes and produces `ClassFile`. Uses `std::io::Cursor` + `Read`. Supports generics parsing via `signature.rs`.

### vm

| Module | Description |
|--------|-------------|
| `oop/` | Object model — `Oop` enum, slot-based heap, instances, arrays, mirrors |
| `runtime/interp/` | Bytecode interpreter (per-opcode files, no macros) |
| `runtime/jit/` | LLVM JIT — bytecode → LLVM IR via stack-to-register conversion |
| `runtime/invoke.rs` | Method dispatch (JIT first, interpreter fallback) |
| `runtime/class_path_manager.rs` | Classpath: directories, JARs, JImage |
| `runtime/class_loader.rs` | Class loading + system dictionary |
| `runtime/thread/` | Thread model, thread pool, Java monitor |
| `native/` | JNI native method implementations (~30 classes) |

### jvm

CLI binary. Initializes VM (oop, runtime, native), resolves classpath, loads and runs the entry class.

### javap

Standalone `javap`-style disassembler. Reads class files and outputs human-readable representations including constant pool, fields, methods, bytecode, line numbers, and stack maps. Uses handlebars templates for rendering.

## Status

| Component | State | Notes |
|-----------|-------|-------|
| Class file parser | Done | nom → Cursor+Read rewrite complete |
| Interpreter | Done | 202/202 opcodes |
| Method invocation | Done | v_table, static, special, interface |
| LLVM JIT | Done | ~110 opcodes, ~110 opcodes translated |
| Oop model | Done | Slot-based, zero unsafe |
| GC | Not implemented | Free-list allocation only, no collection |
| Class verification | Skeleton | No verification implemented |
| invokedynamic | Not implemented | |

## Known Issues

- 5 `class_path_manager` tests fail due to missing `test/` fixture directory
- No garbage collection — objects are allocated but never reclaimed
- `invoke*` bytecodes are not JIT-compiled; they fall back to the interpreter