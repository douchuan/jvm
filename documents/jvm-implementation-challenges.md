# 用 Rust 实现 JVM：语言特性带来的困难与解决方案

> 与 C++ 对比：C++ 是 HotSpot JVM 的实现语言，其灵活性使得大多数 JVM 困难在 C++ 中不存在或很简单。
> 本文从 Rust 语言特性的角度分析每个困难，并与 C++ 的做法对比。

---

## 1. 所有权系统与 GC 的根本冲突

### 困难

Rust 的所有权模型是 **编译期确定、逐层传递** 的：每个值有且仅有一个 owner，生命周期结束时 Drop。

JVM 的 GC 模型是 **运行时动态、图可达性** 的：任意对象可被任意其他对象引用，对象存活由根可达性决定。

```rust
// Rust 的世界观：树形结构
struct Owner {
    child: Box<Child>,  // 唯一所有者，Owner 销毁时 Child 也销毁
}

// JVM 的世界观：图结构
A --ref--> X <--ref-- B    // X 被 A 和 B 同时引用
     `-----> C              // A 销毁时，C 不能被销毁（B 还引用它）
```

**核心矛盾**：Rust 的借用检查器要求每个引用在编译期有确定的生命周期，但 JVM 的引用在运行时可以存活任意久。

- `&T` 引用不能跨越 GC 周期 — GC 可能移动对象，悬垂引用 = UB
- `Rc<T>` / `Arc<T>` 不是 GC — 环形引用导致泄漏，而 Java 对象图中环形引用极常见

### C++ 对比

**C++ 没有这个问题**：

```cpp
// C++：裸指针，编译器不关心生命周期
Oop* a = new Oop();
Oop* b = a;  // 随便拷贝，编译器不管
Oop* c = a;  // 第三个引用，没问题
// 何时 delete？GC 说了算，不用手动管理
```

C++ 的裸指针 `Oop*` 是" dumb pointer"—— 它只是一个地址，没有所有权语义，没有生命周期检查。GC 可以自由移动对象（更新所有指针即可），编译器不会干预。

| | C++ | Rust |
|--|-----|------|
| 对象引用 | `Oop*`，无检查 | `&Oop` 有生命周期，`Box` 唯一所有权 |
| GC 移动对象 | 更新指针即可 | 所有 `&Oop` 悬垂，编译期无法察觉 |
| 环形引用 | 无影响（GC 处理） | `Rc` 泄漏，`&` 不合法 |
| 编译器帮助 | 无 | 有（但阻碍了 GC 模型） |

C++ 的"无帮助"在这里反而是优势——编译器不阻止你做 GC 需要的事。

### 解决方案

**Slot 间接引用**：

```rust
pub struct Heap {
    slots: Vec<Option<Box<dyn OopTrait>>>,
    free_list: Vec<u32>,
}

#[derive(Clone, Copy)]
pub enum Oop {
    Int(i32), Long(i64), Null,
    Ref(u32),  // slot 索引，不是指针也不是引用
}

impl Oop {
    // 访问对象时必须通过 Heap，天然受 Rust 借用检查保护
    pub fn as_ref<'a>(&self, heap: &'a Heap) -> &'a dyn OopTrait {
        heap.get(self.slot_id())
    }
}
```

| 特性 | C++ 裸指针 | Rust Slot 间接 |
|------|-----------|---------------|
| 安全性 | 无（需程序员保证） | 编译期保证 |
| GC 移动对象 | 需更新所有指针 | 只更新 Heap 内部映射 |
| 性能 | 一次直接访问 | 一次间接（多一次查表） |

Slot 间接相当于在 Rust 中模拟了 C++ 裸指针的灵活性，但通过借用检查器保证了安全。

---

## 2. 借用检查器 vs 解释器的操作数栈

### 困难

JVM 解释器的核心是 **操作数栈**，解释器不断 push/pop，类型在编译期不可知。

Rust 要求所有数据访问在编译期确定类型和可变性：

```rust
fn interpret(&mut self) {
    match op {
        IADD => {
            let b = self.stack.pop();  // &mut self borrow
            let a = self.stack.pop();  // 第二次 &mut self — 编译器可能拒绝
            self.stack.push(a + b);    // 第三次
        }
        INVOKESPECIAL => {
            let frame = &mut self.current_frame;
            self.resolve_method(frame);  // frame 被不可变借用
            invoke(frame);               // 又需要可变借用 — 冲突！
        }
    }
}
```

### C++ 对比

**C++ 没有任何借用检查**：

```cpp
void Interpreter::run() {
    switch (opcode) {
        case IADD: {
            auto b = stack.pop_int();
            auto a = stack.pop_int();
            stack.push_int(a + b);  // 想调用几次就几次
            break;
        }
        case INVOKESPECIAL: {
            Frame* frame = current_frame;
            resolve_method(frame);  // 随便用
            invoke(frame);           // 再用一次，编译器不管
            break;
        }
    }
}
```

C++ 中你可以对同一个指针做任意多次可变/不可变操作，编译器不会阻止。这意味着：
- 解释器代码写起来简单直接
- 但也可能在运行时出现 use-after-free、data race 等 UB
- HotSpot 依靠代码审查和测试来保证正确性

Rust 的借用检查器防止了这些 UB，但迫使开发者重写代码结构来满足编译器。

### 解决方案

**单帧独占模型**：

```rust
struct Interpreter<'a> {
    frame: &'a mut Frame,
    heap: &'a mut Heap,
}

impl<'a> Interpreter<'a> {
    fn pop_int(&mut self) -> i32 {
        self.frame.stack.pop_int()
    }
    fn push_int(&mut self, v: i32) {
        self.frame.stack.push_int(v)
    }
    fn run(&mut self) {
        loop {
            match self.frame.next_opcode() {
                IADD => {
                    let b = self.pop_int();
                    let a = self.pop_int();
                    self.push_int(a.wrapping_add(b));
                }
                // 所有操作通过 &mut self 完成，不分裂借用
            }
        }
    }
}
```

关键：每个操作都是对 `&mut self` 的单一借用，内部自行组合，不会暴露中间借用。

---

## 3. 线程同步：Monitor 重入

### 困难

Java Monitor 核心语义：
1. **重入**：同一线程可多次进入同一 monitor
2. **wait/notify**：`Object.wait()` 释放 monitor 并挂起
3. **每个 Java 对象都是 monitor**

Rust 标准库的 `std::sync::Mutex` **不支持重入**—— 第二次 lock 会死锁。

### C++ 对比

**C++ 标准库同样不支持重入**，但 C++ 有更灵活的底层工具：

```cpp
// C++：可以直接用 pthread
pthread_mutexattr_t attr;
pthread_mutexattr_init(&attr);
pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
pthread_mutex_t mutex;
pthread_mutex_init(&mutex, &attr);

// 或者用 Windows CRITICAL_SECTION（本身就支持重入）
// 或者自己包一个计数器
```

C++ 可以直接调用 OS  API，`unsafe` 是隐式的（不需要标记）。Rust 做同样的事需要：
- 用 `unsafe` 封装 `libc::pthread_mutex_t`
- 手动处理 `mem::uninitialized()`（UB 风险）
- 手动实现 `Send`/`Sync` trait

| | C++ | Rust |
|--|-----|------|
| 重入互斥 | pthread / Windows API 直接调用 | 需要 `unsafe` 或第三方 crate |
| 安全性 | 程序员保证 | `parking_lot` 提供安全封装 |
| 平台差异 | 需要 `#ifdef` | crate 跨平台 |

### 解决方案

```rust
use parking_lot::{ReentrantMutex, Condvar};

struct JavaMonitor {
    lock: ReentrantMutex<()>,
    condvar: Condvar,
}
```

零 unsafe，跨平台，比 pthread 快。

---

## 4. 没有运行时类型信息（RTTI）

### 困难

Java 运行时大量依赖类型查询：`instanceof`、`checkcast`、`Object.getClass()`、反射。

Rust 的类型系统在编译期完成，运行时只有 `TypeId`（仅限 `Any` trait）。

### C++ 对比

**C++ 有 RTTI（`dynamic_cast` + `typeid`），但 HotSpot 也不用它**：

```cpp
// C++ RTTI（HotSpot 不用）
Base* obj = getObject();
if (auto derived = dynamic_cast<Derived*>(obj)) { ... }

// HotSpot 的做法（与 Rust 完全一样）
class Klass {
    const char* _name;
    Klass* _super;
    // ...
};
class oopDesc {
    Klass* _klass;  // 每个对象指向类元数据
};
```

**在这个问题上，C++ 和 Rust 完全一样**—— 都需要手工维护运行时类型信息。C++ 的 RTTI 太慢且信息不足，HotSpot 自己实现了一套完整的 Klass 体系。

```rust
// Rust 实现 —— 和 C++ HotSpot 几乎一样
pub struct Class {
    pub name: BytesRef,
    pub super_class: Option<ClassRef>,
    pub interfaces: Vec<ClassRef>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

pub struct InstOopDesc {
    pub class: ClassRef,  // 对应 C++ oopDesc::_klass
    pub fields: Vec<Oop>,
}
```

| | C++ (HotSpot) | Rust |
|--|--------------|------|
| 类型查询 | 手工 Klass 层级 | 手工 Class 结构 |
| 虚方法分发 | vtable | 函数指针 / enum match |
| 对象头 | oopDesc + markOop | InstOopDesc + 字段 |
| 差异 | 裸指针 + vtable | 无 vtable，需手工分派 |

**差异**：C++ 至少还有虚函数表可以做动态分派。Rust 的 `dyn Trait` 也有 vtable，但不适用于 JVM 的动态类加载（运行时新增的类无法在编译期确定 trait 实现）。所以两者最终都用手工表。

---

## 5. 枚举大小爆炸

### 困难

Rust enum 的大小 = **最大变体大小 + discriminant + padding**：

```rust
enum Value {
    Int(i32),              // 只需要 4+1 字节
    Ref(OopRef),           // 24 字节
    Array(Vec<Value>),     // 24 字节
}
// std::mem::size_of::<Value>() = 32 字节（每个变体都占这么多）
```

### C++ 对比

**C++ 没有代数枚举，但有 union**：

```cpp
// C++ union：所有成员共享同一块内存
union Value {
    int32_t i;
    int64_t l;
    float   f;
    double  d;
    Oop*    ref;
    // 大小 = 最大成员 = 8 字节（64 位指针）
};

// HotSpot 的做法：tagged union
class Value {
    enum Tag { INT, LONG, FLOAT, DOUBLE, REF };
    Tag _tag;
    union {
        int32_t i;
        int64_t l;
        float   f;
        double  d;
        Oop*    ref;
    } _data;
};
```

| | C++ union | Rust enum |
|--|-----------|-----------|
| 大小 | 最大成员 | 最大成员 + tag + padding |
| 安全性 | 无（读写不匹配的 member = UB） | 安全（编译器保证） |
| 紧凑性 | 最优 | 相同（编译器优化后） |

实际上 Rust enum 的大小和 C++ tagged union **几乎一样**—— 编译器会自动选择最优的 discriminant 编码。真正的差别在于：

- C++ union 读写不匹配的成员 = UB
- Rust enum match 不完整的分支 = 编译错误

**两者在内存占用上没有显著差异**。

### 解决方案

分层设计，将大对象外置：

```rust
enum Slot {
    Int(i32) = 1,
    Long(i64) = 2,    // 16 字节
    Float(f32) = 3,
    Double(f64) = 4,  // 16 字节
    Ref(u32) = 5,     // 8 字节
}
// 最大 16 字节，远优于内联 Vec/对象的 32+ 字节
```

---

## 6. FFI 与 C 库集成

### 困难

JVM 需要 `JNI_CreateJavaVM` 等 C ABI 导出函数。JNI 方法签名包含可变参数（`va_list`）。

- Rust 的 `c_variadic` 需要 **nightly Rust**
- 跨 FFI 边界的 panic = UB
- 不能通过 FFI 传递 `String`、`Vec` 等 Rust 类型

### C++ 对比

**C++ 天然兼容 C ABI**：

```cpp
// C++：直接 extern "C"，variadic 是语言内置
extern "C" JNIEXPORT jint JNICALL
JNI_CreateJavaVM(JavaVM** pvm, JNIEnv** penv, void* args) {
    return create_jvm(pvm, penv, args);  // 直接调用 C++ 代码
}

extern "C" JNIEXPORT void JNICALL
CallVoidMethod(JNIEnv* env, jobject obj, jmethodID method, ...) {
    va_list args;
    va_start(args, method);
    call_method(env, obj, method, args);  // va_list 直接传递
    va_end(args);
}
```

| | C++ | Rust |
|--|-----|------|
| `extern "C"` | 语言内置，零开销 | `#[no_mangle]` + `extern "C"` |
| 可变参数 | `...` + `va_list`，稳定版支持 | 需要 nightly `c_variadic` |
| panic | 无等价物 | panic 跨 FFI = UB |
| 类型转换 | 隐式（可能有坑） | 必须显式转换 |

**差异**：C++ 的 variadic 是语言内置特性，稳定版即可使用。Rust 的 variadic FFI 还在 nightly，这是一个实际的限制。

### 解决方案

```rust
#[no_mangle]
pub extern "C" fn JNI_CreateJavaVM(
    pvm: *mut *mut c_void,
    penv: *mut *mut c_void,
    args: *mut c_void,
) -> jint {
    unsafe {
        match create_jvm() {
            Ok((vm, env)) => {
                *pvm = Box::into_raw(Box::new(vm)) as *mut c_void;
                *penv = Box::into_raw(Box::new(env)) as *mut c_void;
                JNI_OK
            }
            Err(e) => e.to_jint_code(),
        }
    }
}
```

对于 variadic JNI 函数（`CallVoidMethodV` 等），JNI 规范也提供了非 variadic 的替代版本（`CallVoidMethodA` 接收 `jvalue*` 数组），可以避开 `c_variadic`。

---

## 7. 错误处理：panic vs Java Exception

### 困难

Rust：**二元分流** — `Result<T, E>`（可恢复）vs `panic!`（不可恢复）

Java：**异常是控制流** — 任何方法都可能抛异常，`try-catch` 可捕获并恢复

### C++ 对比

**C++ 有异常，但 HotSpot 也不用**：

```cpp
// HotSpot 内部：禁用 C++ 异常，用返回值传播错误
// -fno-exceptions 编译
jint Interpreter::run() {
    if (obj == NULL) {
        thread->set_pending_exception(vmSymbols::java_lang_NullPointerException());
        return DISPATCH_EXCEPTION;  // 返回码驱动异常传播
    }
    return DISPATCH_NORMAL;
}
```

| | C++ (HotSpot) | Rust |
|--|--------------|------|
| 异常机制 | C++ exception（禁用）+ 返回码 | `Result<T, E>` |
| 控制流异常 | 返回 `DISPATCH_EXCEPTION` | 返回 `Err(JvmError)` |
| unwind 支持 | 有但禁用 | `catch_unwind`（有开销） |

**在这个问题上，Rust 的 `Result` 比 C++ exception 更适合 JVM 实现**：

- 不需要 `-fno-exceptions` 来优化性能
- `Result` 的零开销抽象（没有异常就无额外开销）
- 编译器强制调用方处理错误

### 解决方案

```rust
enum JvmError { NullPointer, ArrayIndexOutOfBounds, ClassCast, ... }
type JvmResult<T> = Result<T, JvmError>;

fn run(&mut self) -> JvmResult<()> {
    match self.execute_opcode(op) {
        Ok(()) => {},
        Err(e) => match self.frame.find_handler(&e) {
            Some(pc) => { self.frame.pc = pc; }  // 继续执行
            None => return Err(e),               // 传播给调用者
        },
    }
}
```

---

## 8. 无稳定 ABI 与动态类加载

### 困难

Rust 没有稳定的 ABI：struct 布局不固定，不同编译器版本不兼容。

### C++ 对比

**C++ 也没有稳定的 ABI**（不同编译器、不同版本之间），但这不影响 JVM 实现：

```cpp
// 动态类加载不是在加载动态库
// 而是在进程内构造数据结构
Klass* ClassLoader::load_class(const char* name) {
    // 读取 .class 文件
    // 在堆上构造 Klass 对象
    // 注册到 SystemDictionary
}
```

| | C++ | Rust |
|--|-----|------|
| struct 布局 | 不保证（除非 `#pragma pack`） | 不保证（除非 `#[repr(C)]`） |
| 动态链接 | `dlopen` + 符号查找 | `libloading` crate |
| 类加载 | 进程内构造数据结构 | 进程内构造数据结构 |
| 差异 | 无 | 无 |

**结论**：JVM 的动态类加载是进程内的数据结构构造，与语言的 ABI 无关。C++ 和 Rust 在这个问题上没有差异。

---

## 9. 宏与代码生成

### 困难

200+ 条字节码指令。Rust 宏可以减少重复，但宏展开后的错误信息难以阅读。

### C++ 对比

**C++ 有同样强大的宏系统和模板元编程**：

```cpp
// C++ 宏（OpenJDK 大量使用）
#define GENERATE(code, name, x, y, z) \
  case Bytecodes::_##name:            \
    return Interpreter::_##name;

BYTECODES(GENERATE)

// C++ 模板元编程（编译期计算）
template<typename T>
struct InterpreterRuntime {
    static void resolve(Method* m) { ... }
};
```

| | C++ | Rust |
|--|-----|------|
| 宏 | 文本替换（无卫生） | 卫生宏（TT-Muncher） |
| 编译期计算 | 模板元编程（图灵完备） | const generics + trait bounds |
| 代码生成 | 宏 + 模板 | 宏 + 过程宏 |
| 错误信息 | 模板错误极难读懂 | 宏错误比 C++ 好一点 |
| HotSpot 实际做法 | 大量用宏生成 opcode 表 | — |

**两者都有强大的编译期代码生成能力**，也都容易写出难以调试的元代码。HotSpot 的做法是用宏生成 opcode 分发表—— 这和 Rust 用宏生成函数指针表完全对应。

### 解决方案

保持显式，不用宏：

```rust
// 每条指令一个函数，5-10 行
fn op_iadd(&mut self) {
    let b = self.pop_int();
    let a = self.pop_int();
    self.push_int(a.wrapping_add(b));
}
```

---

## 总结对比

| 困难 | C++ 体验 | Rust 体验 | 根本原因 |
|------|---------|----------|---------|
| GC / 对象引用 | 裸指针，编译器不干预 | 借用检查器阻止 | Rust 的安全保证与 GC 模型冲突 |
| 解释器栈操作 | 随便读写，零约束 | 需满足借用规则 | 编译期 vs 运行时可变性 |
| Monitor 重入 | 调 pthread/Win API | 需要 unsafe 或第三方 crate | 标准库功能差异 |
| 运行时类型信息 | RTTI 不用，手工维护 | 一样手工维护 | **无差异** |
| Enum/Union 大小 | union 紧凑 | enum 同样紧凑 | **无差异** |
| JNI FFI | 天然兼容 C | variadic 需 nightly | C++ 更接近 C |
| 异常处理 | exception 禁用，用返回码 | Result 天然适配 | **Rust 更好** |
| 动态类加载 | 进程内构造 | 一样 | **无差异** |
| 代码生成 | 宏 + 模板 | 卫生宏 | 各有优劣 |

**核心结论**：

1. **C++ 最大的灵活性来自"编译器不阻止你做危险的事"**—— 裸指针随意拷贝、内存任意布局、FFI 无缝。这些在 JVM 实现中确实是优势。

2. **Rust 的约束在 GC 和操作数栈上最明显**—— 但通过 Slot 间接引用和单帧独占模型可以优雅解决。

3. **在异常处理上 Rust 反而优于 C++**—— `Result` 比 C++ exception（需要 `-fno-exceptions` 优化）更适合 JVM 的控制流异常模型。

4. **Runtime type info、ABI、Enum 大小等问题上，两者没有实质差异**—— HotSpot 也不依赖 C++ 的这些特性。
