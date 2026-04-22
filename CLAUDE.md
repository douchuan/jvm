# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A JVM implementation written in Rust — a learning project aiming for a complete JVM with LLVM JIT and GC. Actively being refactored, supports JDK 9+ (JImage class loading).

## Workspace Structure

```
jvm/                     # Binary crate — CLI entry point (main.rs + options.rs)
crates/
  classfile/             # JVM class file format type definitions (zero dependencies)
  class-parser/          # Parser: bytes -> ClassFile (Cursor + Read)
  vm/                    # Core VM: interpreter, Oop model, native methods, threading, JIT
  class-verification/    # Class verification (skeleton)
tools/
  javap/                 # Class file disassembler tool
tests/
  java/                  # Java test suite — 17 files covering arithmetic, OOP, arrays, exceptions, etc.
scripts/
  dev.sh                 # Convenience script for common tasks
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

### Dependency management

All dependencies are centralized in the root `Cargo.toml` under `[workspace.dependencies]`.
Sub-crates reference them with `dep.workspace = true`. To add or upgrade a dependency,
edit only the root `Cargo.toml`.

## Key Commands

```bash
# Quick reference — use dev.sh for convenience
./scripts/dev.sh build              # Build workspace (debug)
./scripts/dev.sh build-release      # Build workspace (release)
./scripts/dev.sh run <Class> [args] # Run a Java class
./scripts/dev.sh test               # Run all tests
./scripts/dev.sh javap <classfile>  # Disassemble a class file
./tests/java/run.sh                 # Compile & run Java test suite against the JVM
./scripts/dev.sh clean              # Clean build artifacts

# Or use cargo directly
cargo build                   # Builds all workspace members
cargo build --workspace       # Same as above (explicit)
cargo test --workspace
cargo test -p class-parser hello_world
cargo run -p jvm -- <ClassName>
cargo run -p javap -- <classfile>
```

### Class parser test fixtures

Java source files live in `crates/class-parser/tests/fixtures/src/` and are compiled to `.class` at build time via `build.rs`. Do **not** commit `.class` files — only `.java` sources.

### Java test suite

`tests/java/src/` contains 17 Java files covering the JVM's supported features. Each targets a specific domain (arithmetic, OOP, arrays, exceptions, etc.). Run with `./tests/java/run.sh`. Do **not** commit `.class` files.

## Architecture

### classfile (`crates/classfile/`)

Pure type definitions for the JVM class file format. No parsing logic. Core types:
- `ClassFile` — top-level struct
- `ConstantPool` / `ConstantPoolType` — constant pool entries
- `MethodInfo` / `FieldInfo` — method and field descriptors
- `Attribute` variants — Code, LineNumberTable, Exceptions, etc.
- `Opcode` — all 200+ JVM bytecode instructions

### class-parser (`crates/class-parser/`)

Parses raw bytes into `ClassFile`. Uses `std::io::Cursor` + `Read` (no nom). Files:
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
- `jit/` — LLVM JIT compiler (inkwell):
  - `mod.rs` — JIT compiler lifecycle (thread-local)
  - `builder.rs` — bytecode → LLVM IR translation (~110 opcodes)
  - `ops.rs` — runtime callouts for complex operations (new, invoke, field access)
- `frame.rs` / `stack.rs` / `slot.rs` / `local.rs` — call stack structures
- `invoke.rs` — method invocation logic (JIT first, interpreter fallback)
- `method.rs` — method representation
- `class_loader.rs` — class loading from classpath
- `class_path_manager.rs` — classpath management (DIR, JAR, JImage)
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

## Development Roadmap

### Phase 1: JIT 编译器完善（优先级：高）
- JIT 方法调用：`invokevirtual`/`invokespecial`/`invokestatic`/`invokeinterface` 的 LLVM IR 翻译
  - 策略：通过 runtime callout（`jit/ops.rs`）回退到解释器路径
- JIT 控制流：完善 `tableswitch`、`lookupswitch`、异常处理路径
- JIT 返回值扩展：当前仅支持 int，需支持 long/float/double/ref
- JIT 参数类型扩展：`copy_args_to_locals` 当前仅处理 `Oop::Int`

### Phase 2: 垃圾回收（优先级：高）
- 目标：Mark-Sweep-Compact
- Root 收集：遍历线程栈（locals + stack）+ 静态字段
- Mark/Sweep/Compact 三阶段
- 触发策略：分配失败时 full GC → 后续引入分代

### Phase 3: 类验证器（优先级：中）
- 文件：`crates/class-verification/`（已有骨架）
- 两阶段：结构性验证 + 字节码类型安全验证

### Phase 4: invokedynamic（优先级：中）
- BootstrapMethods 属性解析 + CallSite 链接机制
- `java.lang.invoke.MethodHandle` native 方法

### Phase 5: 工程质量（持续）
- 修复 5 个 class_path_manager 测试（创建 test/ fixtures）
- 清理 javap warning（未使用导入、dead_code trait 方法）
- handlebars 替换预案（纯字符串拼接 或 tera）
- 测试覆盖率基线（cargo-llvm-cov）

## Current Phase Status

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Class-parser rewrite | Done | nom → Cursor+Read, 12 tests |
| 2. Workspace compilation | Done | `cargo build` passes, all members compile together |
| 3. Test skeleton | Done | 34 tests pass, 5 pre-existing failures (missing fixtures) |
| 4. Oop model rewrite | Done | Slot-based (`Oop::Ref(u32)`), zero unsafe |
| 5. Interpreter rewrite | Done | Per-opcode files, 202/202 opcodes |
| 6. Method invocation | Done | hack_as_native, v_table fix |
| 7. LLVM JIT | Partial | ~110 opcodes (int/long/float/double arith, bitwise, stack ops, conversions). invoke* not JIT-compiled |
| 8. GC precise-ification | Planned | slot-based heap → mark-sweep-compact |
| 9. Complete features | Planned | invokedynamic, verification, threading |

See `documents/jvm-implementation-challenges.md` for detailed Rust vs C++ JVM implementation analysis.

## Known Issues

- **5 class_path_manager tests fail** — missing `test/` fixture directory (pre-existing, not a functional bug)
- **No GC** — objects allocated via free-list but never collected
- **Class verification skeleton** — 0 implementations
- **invoke\* not in JIT** — method calls fall back to interpreter
- **handlebars 4.5 compatibility** — javap 依赖 handlebars，若新版 Rust 编译失败可考虑替换为 `tera` 或纯模板拼接

## Important Conventions

- **Oop model**: `Oop::Ref(u32 slot_id)` with `Heap` indirection — zero unsafe code for object access. Use `oop::with_heap(|heap| ...)` and `oop::with_heap_mut(|heap| ...)` to access heap objects
- **Class model**: `ClassRef = Arc<Class>`, mutable fields use `RwLock` internally, accessed via accessor methods
- **Monitor**: `RefKindDesc` uses safe `std::sync::{Mutex, Condvar}` for Java monitor semantics
- **Exception handling**: `Result<T, JvmError>` — never panic for Java exceptions
- **No comments** unless the WHY is non-obvious (hidden constraint, workaround for specific bug)
