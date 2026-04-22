# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A JVM implementation written in Rust — a learning project aiming for a complete JVM with LLVM JIT and GC. Actively being refactored, supports JDK 9+ (JImage class loading).

## Workspace Structure

```
jvm/                     # Binary crate — CLI entry point (main.rs + options.rs)
crates/
  classfile/             # JVM class file format type definitions (zero dependencies)
  class-parser/          # Parser: bytes -> ClassFile (Cursor + Read, edition 2021)
  vm/                    # Core VM: interpreter, Oop model, native methods, threading
  class-verification/    # Class verification (skeleton)
tools/
  javap/                 # Class file disassembler tool (standalone)
documents/
  jvm-implementation-challenges.md  # Rust vs C++ JVM implementation difficulties
```

### Dependency graph

```
jvm (binary) → vm → class-parser → classfile
                          ↘ classfile
javap (tool) → class-parser → classfile
                  ↘ classfile
```

## Key Commands

```bash
# Build entire workspace
cargo build --workspace

# Run tests (class-parser has 23 tests: 12 unit + 11 integration)
cargo test --workspace

# Run a specific test
cargo test -p class-parser hello_world

# Run the JVM binary
./target/debug/jvm <ClassName>

# Run javap disassembler
./target/debug/javap <classfile>
```

### Class parser test fixtures

Java source files live in `crates/class-parser/tests/fixtures/src/` and are compiled to `.class` at build time via `build.rs`. Do **not** commit `.class` files — only `.java` sources.

## Architecture

### classfile (`crates/classfile/`)

Pure type definitions for the JVM class file format. No parsing logic. Core types:
- `ClassFile` — top-level struct
- `ConstantPool` / `ConstantPoolType` — constant pool entries
- `MethodInfo` / `FieldInfo` — method and field descriptors
- `Attribute` variants — Code, LineNumberTable, Exceptions, etc.
- `Opcode` — all 200+ JVM bytecode instructions

### class-parser (`crates/class-parser/`)

Praws raw bytes into `ClassFile`. Uses `std::io::Cursor` + `Read` (no nom). Files:
- `reader.rs` — `Reader` struct with `read_u8/u16/u32/bytes/utf8`
- `constant_pool.rs` — constant pool parsing
- `attributes.rs` — attribute parsing (Code, Signature, etc.)
- `fields.rs` / `methods.rs` — field/method parsing
- `class.rs` — top-level `parse_class_file` assembly
- `signature.rs` — method/field signature parsing (generics, arrays, etc.)

Entry point: `parse(&[u8]) -> Result<ClassFile>` or `parse_class` (alias).

### vm (`crates/vm/`)

The core VM. Major modules:

**oop/** — Object model
- `mod.rs` — `Oop` enum, slot-based heap access (`with_heap`/`with_heap_mut`)
- `heap.rs` — slot-based object allocation (`Oop::Ref(u32)`)
- `reference.rs` — `RefKind`, `RefKindDesc`, safe `Monitor` (std Mutex+Condvar)
- `class.rs` / `inst.rs` / `ary.rs` / `field.rs` — object kinds

**runtime/** — Execution engine
- `interp/` — interpreter, split into per-opcode files (no macros):
  - `const_ops.rs` / `load_store.rs` / `arith_ops.rs` / `stack_ops.rs` / `conversion.rs`
  - `control_flow.rs` / `compare.rs` / `object_ops.rs` / `field_ops.rs` / `array_ops.rs`
  - `monitor_ops.rs` / `exception.rs` / `read.rs`
- `frame.rs` / `stack.rs` / `slot.rs` / `local.rs` — call stack structures
- `invoke.rs` — method invocation logic
- `method.rs` — method representation
- `class_loader.rs` — class loading from classpath
- `thread/` — Java threads, mutex, condvar, thread pool
- `vm.rs` — VM initialization and lifecycle

**native/** — JNI native method implementations
- `java_lang_*` — Object, String, System, Thread, Class, etc.
- `sun_misc_*` — Unsafe, VM, Signal, etc.
- `sun_reflect_*` — reflection support

### jvm (`jvm/`)

Binary crate — CLI entry point. Parses command-line options, initializes VM, runs the target class.

### javap (`tools/javap/`)

Standalone class file disassembler. Outputs `javap`-style human-readable class dumps. Uses handlebars for template rendering.

## Current Refactoring Plan

The project is being rewritten from scratch in phases. Progress:

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Class-parser rewrite | Done | nom → Cursor+Read, 12 tests |
| 2. Workspace compilation | Done | `cargo build --workspace` passes |
| 3. Test skeleton | Done | 23 tests (12 unit + 11 integration) |
| 4. Oop model rewrite | Done | Slot-based (`Oop::Ref(u32)`), zero unsafe |
| 5. Interpreter rewrite | Done | Per-opcode files, no macros |
| 6. Method invocation | Done | Adapt to new Frame/Stack |
| 7. LLVM JIT | Next | inkwell + alloca/mem2reg |
| 8. GC precise-ification | Planned | slot-based heap → mark-sweep-compact |
| 9. Complete features | Planned | invokedynamic, verification, threading |

See `documents/jvm-implementation-challenges.md` for detailed Rust vs C++ JVM implementation analysis.

## Important Conventions

- **Oop model**: `Oop::Ref(u32 slot_id)` with `Heap` indirection — zero unsafe code for object access. Use `oop::with_heap(|heap| ...)` and `oop::with_heap_mut(|heap| ...)` to access heap objects
- **Class model**: `ClassRef = Arc<Class>`, mutable fields use `RwLock` internally, accessed via accessor methods
- **Monitor**: `RefKindDesc` uses safe `std::sync::{Mutex, Condvar}` for Java monitor semantics
- **Exception handling**: `Result<T, JvmError>` — never panic for Java exceptions
- **No comments** unless the WHY is non-obvious (hidden constraint, workaround for specific bug)
