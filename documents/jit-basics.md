# JVM JIT 编译器基本原理

## 为什么需要 JIT

解释器逐字节码执行，每次循环都要：取字节 → 查 OpCode → match 分发 → 执行。热点方法被调用百万次后，这个开销显著。JIT 在运行时将字节码翻译为本地机器码，消除了解释器的 dispatch overhead，并允许 LLVM 做寄存器分配、循环优化等。

```
解释器:  bytecode → dispatch loop → 每次解释执行
  JIT  :  bytecode → LLVM IR → 机器码 → 直接执行
```

## 编译 Pipeline

```
ClassFile.method.code
         ↓
    [Bytecode Reader]
         ↓ 逐条 opcode 翻译
    [LLVM IR Builder]
         ↓ BasicBlock per 跳转目标
    [LLVM IR Module]
         ↓ Optimization passes (mem2reg, instcombine, ...)
    [ExecutionEngine]
         ↓ 编译为机器码
    fn pointer → 缓存到 MethodId.jit_impl
```

## 核心概念

### BasicBlock

LLVM 的 `BasicBlock` 是无标签 fallthrough 的指令序列。每个 JVM 字节码位置映射到一个 BasicBlock，控制流指令（goto、if*、tableswitch）映射为 `br`/`condbr`/`switch`：

```
字节码:
  0: iload_0
  1: iconst_1
  2: iadd
  3: ifeq 7
  6: ireturn
  7: goto 0

LLVM IR:
  bb_0:  %v0 = load i32, ptr %local_0
         %v1 = add i32 %v0, 1
         %c  = icmp eq i32 %v1, 0
         br i1 %c, label %bb_7, label %bb_6
  bb_6:  ret i32 %v1
  bb_7:  br label %bb_0
```

### Stack-to-Register 转换

JVM 是基于栈的机器：

```java
int add(int a, int b) { return a + b; }
// iload_0, iload_1, iadd, ireturn
```

解释器维护一个 operand stack，push/pop 操作它。但 LLVM IR 是 SSA 形式（Static Single Assignment），使用虚拟寄存器。转换策略：

```
// 方法开始时
let stack_vars = vec![alloca_i32(); max_stack];
let mut stack_ptr = 0;

// push_int(v) → stack_vars[stack_ptr] = v; stack_ptr += 1;
// pop_int()   → stack_ptr -= 1; stack_vars[stack_ptr]

// LLVM mem2reg pass 会将这些 alloca 提升为 SSA 寄存器
```

关键：LLVM 的 `mem2reg`（PromoteMemoryToRegister）pass 自动消除 alloca 的内存访问，将局部变量提升为 SSA register，所以不需要手动做 SSA 构造。

### Locals 处理

JVM 的 local variables 数组在编译期已知大小（`Code.max_locals`）。MVP 策略：

```
每个 local slot → LLVM alloca
iload_0  → load from alloca_i32 %local_0
istore_1 → store to alloca_i32 %local_1
```

mem2reg 会自动优化为寄存器操作。对于 long/double 占 2 个 slot 的情况，编译器需正确处理 slot 偏移（`step=2`）。

### 控制流

| JVM 指令 | LLVM IR 映射 |
|----------|-------------|
| `goto offset` | `br label %bb_target` |
| `ifeq offset` | `condbr i1 %eq, label %bb_true, label %bb_fall` |
| `if_icmpgt offset` | `cmp + condbr` |
| `tableswitch` | `switch i32 %val, label %default [...]` |
| `lookupswitch` | 同上（LLVM 的 switch 自动优化为二分/跳转表） |
| `return` | `ret <type> %value` 或 `ret void` |

## 调用边界

JVM 程序中存在解释方法和编译方法混合执行的情况。边界在方法调用处：

```
invoke_java():
  if method.jit_impl.is_some():
    jit_fn = method.jit_impl.unwrap()
    jit_fn(&locals, &stack)  // 直接调用编译后的函数
  else:
    Interp::new(frame, local).run()  // 解释执行
```

`jit_impl` 与 `native_impl` 模式一致——都是 `Option<fn pointer>`，一个是 native 代码，一个是 JIT 编译代码。

## 运行时调用（Runtime Callout）

编译后的代码不能直接操作 Java 堆对象，需要通过运行时函数：

| 操作 | 运行时函数 |
|------|-----------|
| `new` 对象 | `Oop::new_inst(cls) → Oop::Ref(u32)` |
| `getfield` | `Class::get_field_value2(slot_id, offset) → Oop` |
| `putfield` | `Class::put_field_value2(slot_id, offset, value)` |
| 方法调用 | `JavaCall::invoke()` |
| 类型检查 | `cmp::instance_of(obj_cls, target_cls) → bool` |
| 异常 | `exception::meet_ex(cls_name, msg)` |
| 锁 | `RefKindDesc::monitor_enter/exit()` |

JIT 生成的 IR 中，这些函数通过 `module.add_function()` 声明为 external，LLVM 执行时自动链接。

## 槽位栈 vs 寄存器优化

解释器的栈模型（`Vec<Slot>`）是动态的，但 JIT 编译时可以静态确定栈深度：

```
// 解释器：运行时 push/pop，边界检查
stack.push_int(v);  // Vec::push，可能 realloc

// JIT：编译期已知 max_stack，预分配固定大小
let stack_base = alloca [max_stack x i64];  // 连续内存
// 或更激进：用一组独立变量，mem2reg 自动优化
```

实际实现中，更简单的做法是用 LLVM 变量跟踪栈指针，每个 push/pop 对应变量读写。LLVM 优化后等同于寄存器操作，没有内存访问开销。

## 异常处理

JVM 的 exception table 是 PC 范围映射：

```
struct CodeException {
    start_pc: u16, end_pc: u16,  // 受保护的 PC 范围
    handler_pc: u16,             // 异常处理入口
    catch_type: u16,             // 捕获的异常类
}
```

解释器每次 opcode 执行后检查 `thread::is_meet_ex()`，然后遍历 exception table。JIT 编译时需要：
1. 在每个可能抛出异常的指令后插入检查
2. 维护 PC 到 LLVM basic block 的映射
3. 异常时查表跳转到 handler basic block

MVP 阶段可先跳过异常处理，只编译无异常的方法。

## 惰性编译（Lazy Compilation）

最简单的触发策略：首次调用时编译。

```
第一次调用 method:
  1. 检查 method.jit_impl → None
  2. 调用 JitCompiler::compile(method)
  3. 编译成功 → method.jit_impl = Some(fn_ptr)
  4. 调用 fn_ptr

后续调用:
  1. 检查 method.jit_impl → Some(fn_ptr)
  2. 直接调用 fn_ptr
```

更高级的策略是调用计数器（热点方法才编译），但增加了复杂度。MVP 用惰性编译即可。

## LLVM 优化 Pass

inkwell 的 ExecutionEngine 默认启用优化。关键 pass：

| Pass | 作用 |
|------|------|
| mem2reg | alloca → SSA register，消除内存访问 |
| instcombine | 常量折叠、死代码消除 |
| simplifycfg | 合并基本块、消除不可达分支 |
| GVN | 全局值编号，消除冗余 load |
| LICM | 循环不变代码外提 |

对于简单的 `int add(int a, int b)`，mem2reg + instcombine 后 LLVM 可能直接内联为常量或单条加法指令。

## 当前架构适配

### MethodId 存储

```rust
// crates/vm/src/runtime/method.rs
pub struct MethodId {
    pub method: Method,
    pub native_impl: Option<JNINativeMethod>,
    pub jit_impl: Option<JITCompiledMethod>,  // 新增
}
```

### 编译函数签名

```rust
// 返回值通过 locals 区域的 return slot 传递
// 或通过 stack 的 top 元素传递
type JITFn = extern "C" fn(locals: *mut Slot, stack: *mut Slot);
```

### 入口点

在 `invoke_java` 中插入编译检查：

```rust
// crates/vm/src/runtime/invoke.rs, invoke_java()
if let Some(jit_fn) = self.mir.jit_impl {
    jit_fn(/* locals ptr, stack ptr */);
} else {
    // 可选：先编译再执行
    // 或直接走解释器
    let mut interp = Interp::new(frame_h, local);
    interp.run();
}
```
