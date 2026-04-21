// ============================================================
// JIT 编译器模块
//
// 本模块负责将 JVM 字节码编译为 LLVM IR，然后 JIT 编译为本地机器码。
//
// ## LLVM 核心概念
//
// **LLVM** 是一个编译器基础设施，分为前端（生成 IR）、优化器（优化 IR）、
// 后端（生成机器码）。我们的 JIT 使用 LLVM 的前端 + 后端，跳过磁盘文件阶段。
//
// **IR（Intermediate Representation）** 是 LLVM 的中间表示，一种类汇编的
// 三地址码格式。例如：`%sum = add i32 %a, %b`
//
// **BasicBlock** 是无条件跳转目标的指令序列。每个 basic block 以终止符
//（br/ret/switch）结尾。JVM 的每个跳转目标对应一个 LLVM BasicBlock。
//
// **ExecutionEngine** 是 inkwell 提供的运行时编译+执行组件。它接受 LLVM Module，
// 编译为机器码，并提供函数指针供我们直接调用。
//
// **mem2reg** 是 LLVM 最重要的优化 pass。它将 `alloca`（栈内存分配）提升为
// SSA register。所以我们不需要手动做 SSA 构造——只管 alloca + store/load，
// LLVM 自动优化。
//
// ## 为什么用 thread_local
//
// inkwell 的 Context、Module、Builder、ExecutionEngine 不是 Send/Sync 的。
// 这意味着它们不能在线程间安全传递或共享。原因是 LLVM 内部使用了很多
// 非线程安全的数据结构（裸指针、可变全局状态等）。
//
// 解决方案：每个线程拥有自己的 JIT 编译器实例。
// 这与 JVM 的线程模型一致——每个 Java 线程对应一个 OS 线程。
//
// ## 编译流程
//
// bytecode → LLVM IR (builder.rs + ops.rs)
//         → ExecutionEngine 编译为机器码
//         → 返回函数指针 → 缓存到 MethodId.jit_impl
// ============================================================

use crate::runtime::method::{JITCompiledMethod, JitFn};
use crate::types::MethodIdRef;
use std::cell::RefCell;
use std::sync::Arc;

mod builder;
mod ops;

// inkwell 的 Context 是 LLVM 的上下文对象，持有所有 LLVM 类型、值、模块的
// 所有权。它是 JIT 编译器的"根"。
//
// 重要：Context 的生命周期必须长于所有由它创建的 LLVM 对象。
// 所以我们把它放在 JitCompiler 结构体中，每个线程一个实例。
use inkwell::context::Context;

// ExecutionEngine 负责将 LLVM IR 编译为可执行的机器码。
// JIT 模式下，编译后的代码在内存中可直接调用。
use inkwell::execution_engine::ExecutionEngine;

// Builder 是 LLVM IR 的"指令发射器"。你在当前插入点（insertion point）
// 调用 builder.add_function() 等方法，它生成对应的 LLVM IR 指令。
use inkwell::builder::Builder;

// Module 是 LLVM IR 的容器，相当于一个"编译单元"。
// 一个 Module 可以包含多个函数，被 ExecutionEngine 编译。
use inkwell::module::Module;

// OptimizationLevel 用于控制 JIT 编译器的优化级别。
// None = 无优化（编译最快），Aggressive = 最大优化（执行最快）。
// MVP 使用 None，后续可以改为 Aggressive。
use inkwell::OptimizationLevel;

/// JIT 编译器主结构体。
///
/// 每个线程拥有一个 JitCompiler 实例。它持有：
/// - `context`: LLVM 上下文，所有 LLVM 对象的父级
/// - `module`: LLVM IR 模块，所有 JIT 编译的函数都在这里
/// - `builder`: IR 指令构建器，用于逐个方法生成 IR
/// - `execution_engine`: 运行时编译器，将 IR → 机器码
///
/// 注意：这些类型都不是 Send/Sync，所以不能跨线程共享。
/// 使用 RefCell 包裹是因为同一个线程内我们需要可变借用。
pub struct JitCompiler {
    /// LLVM 上下文。使用 `'static` 引用，通过 `Box::leak` 确保
    /// 上下文在整个程序生命周期内有效。
    _context: &'static Context,
    /// LLVM IR 模块。所有编译的方法函数都在这里定义。
    module: Module<'static>,
    /// IR 指令构建器。每次编译一个方法时，用它发射 IR 指令。
    builder: Builder<'static>,
    /// 运行时执行引擎。负责将 LLVM Module 编译为本地机器码。
    execution_engine: ExecutionEngine<'static>,
}

impl JitCompiler {
    /// 创建新的 JIT 编译器实例。
    ///
    /// 初始化步骤：
    /// 1. 创建 `Context` —— LLVM 的根对象
    /// 2. 创建 `Module` —— 命名为 "jvm_jit_module"
    /// 3. 创建 `Builder` —— 用于发射 IR 指令
    /// 4. 创建 `ExecutionEngine` —— 启用优化
    ///
    /// 如果 LLVM 初始化失败（如版本不匹配），返回 `Err`。
    pub fn new() -> Result<Self, String> {
        // 创建 LLVM 上下文。这是所有后续 LLVM 操作的根。
        let context = Context::create();

        // 关键：我们需要将 context 存储到结构体中，但 module 和 builder
        // 内部引用了 context。在 Rust 中这是自引用结构体问题。
        // 解决方案：使用 `Box::leak` 将 context 放到堆上并泄漏引用，
        // 这样 context 的地址在整个程序生命周期内有效。
        // 必须在创建 module 之前 leak，否则 module 会持有旧 context 的引用。
        let leaked = Box::leak(Box::new(context));
        let module = leaked.create_module("jvm_jit_module");

        // 创建 Builder。Builder 负责在 BasicBlock 中插入 LLVM IR 指令。
        // 类比：Builder 就像汇编器，你告诉它"生成 add 指令"，它就生成。
        let builder = leaked.create_builder();

        // 创建 ExecutionEngine。使用 JIT 编译器模式（而非解释器模式）。
        // create_jit_execution_engine 会设置 jit_mode = true，
        // 使得 get_function 可以获取编译后的函数指针。
        // create_execution_engine 默认 jit_mode = false，不能用 get_function。
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT execution engine: {}", e))?;

        Ok(Self {
            _context: leaked,
            module,
            builder,
            execution_engine,
        })
    }

    /// 编译一个 Java 方法的字节码为本地机器码。
    ///
    /// ## 编译步骤
    ///
    /// 1. 检查方法是否有 bytecode（native/abstract 方法没有）
    /// 2. 在 LLVM Module 中创建一个新函数
    /// 3. 为每个字节码位置创建 BasicBlock
    /// 4. 从入口 BasicBlock 开始，逐条翻译 bytecode → LLVM IR
    /// 5. ExecutionEngine 将函数编译为机器码
    /// 6. 返回函数指针，缓存到 MethodId.jit_impl
    ///
    /// ## 函数签名
    ///
    /// 生成的 LLVM 函数签名是：`void fn(i8* locals, i8* stack)`
    /// - `locals`: 本地变量数组的起始指针
    /// - `stack`: 操作数栈的起始指针
    pub fn compile_method(&mut self, method_id: &MethodIdRef) -> Option<Arc<JITCompiledMethod>> {
        // 获取方法的 bytecode。native 方法和 abstract 方法没有 bytecode，
        // 不需要（也不能）编译。
        let code = method_id.method.code.as_ref()?;
        let bytecode = code.code.as_slice();
        let max_locals = code.max_locals as usize;
        let max_stack = code.max_stack as usize;

        // 调用 builder 模块进行实际的 IR 生成。
        // 注意：我们将 context 的引用传递给 builder，因为 LLVM IR 的
        // 创建（如 BasicBlock、常量等）需要 context。
        let function = builder::compile_method(
            self._context,
            &self.module,
            &self.builder,
            &method_id.method,
            bytecode,
            max_locals,
            max_stack,
        )?;

        // 验证生成的 IR 是否正确。
        // LLVM 的 verify_function 检查 IR 的合法性：
        // - 每个 BasicBlock 都以终止符结尾
        // - 类型匹配
        // - PHI 节点位置正确
        if !function.verify(true) {
            warn!(
                "JIT: LLVM IR verification failed for method {:?}",
                method_id.method
            );
            return None;
        }

        // 通过 ExecutionEngine 查找编译后的函数指针。
        // get_function 会自动触发 JIT 编译（如果尚未编译）。
        let fn_name = function.get_name().to_str().unwrap().to_string();
        let fn_ptr: JitFn = unsafe {
            self.execution_engine
                .get_function(&fn_name)
                .map(
                    |f: inkwell::execution_engine::JitFunction<
                        unsafe extern "C" fn(*mut (), *mut ()),
                    >| {
                        // as_raw() 返回底层 unsafe 函数指针。
                        // 我们需要将其转换为安全的 JitFn 类型（extern "C" fn(*mut i32, *mut i32)）。
                        // 这是安全的，因为 JIT 编译的代码确实是一个有效的 C ABI 函数。
                        std::mem::transmute::<unsafe extern "C" fn(*mut (), *mut ()), JitFn>(
                            f.as_raw(),
                        )
                    },
                )
                .map_err(|e| {
                    warn!("JIT: Failed to get compiled function pointer: {:?}", e);
                })
                .ok()?
        };

        Some(Arc::new(JITCompiledMethod { fn_ptr }))
    }
}

/// 线程局部的 JIT 编译器实例。
///
/// 每个线程拥有自己的 Context + Module + ExecutionEngine。
/// 使用 RefCell 是因为我们在同一线程内需要可变借用（编译方法会修改 Module）。
thread_local! {
    static JIT_COMPILER: RefCell<Option<JitCompiler>> = const { RefCell::new(None) };
}

/// 尝试编译一个方法。
/// 如果 JIT 编译器不可用、未初始化或编译失败，返回 None。
/// 调用方应回退到解释器执行。
pub fn try_compile(method_id: &MethodIdRef) -> Option<Arc<JITCompiledMethod>> {
    JIT_COMPILER.with(|cell| {
        let mut guard = cell.borrow_mut();
        let compiler = guard.as_mut()?;
        compiler.compile_method(method_id)
    })
}

/// 初始化当前线程的 JIT 编译器。
///
/// 必须在调用 `try_compile` 之前调用一次。
/// 如果 LLVM 初始化失败，返回 Err，JIT 功能不可用（但解释器仍可工作）。
/// 多次调用是安全的——已初始化的线程会直接返回 Ok。
pub fn init() -> Result<(), String> {
    JIT_COMPILER.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_some() {
            return Ok(()); // 已初始化
        }
        let compiler = JitCompiler::new()?;
        *guard = Some(compiler);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    use inkwell::types::BasicType;

    /// 端到端测试：用 LLVM 创建并执行一个简单函数。
    ///
    /// 测试内容：
    /// 1. LLVM 初始化是否正常
    /// 2. 函数创建 → IR 生成 → ExecutionEngine 编译 → 执行 的完整链路
    #[test]
    fn test_llvm_pipeline() {
        // 创建最小 LLVM 环境（类似 JIT 初始化流程）
        let context = Context::create();
        let leaked = Box::leak(Box::new(context));
        let module = leaked.create_module("test_module");
        let builder = leaked.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .expect("Failed to create execution engine");

        // 创建函数: i32 fn(i32* a, i32* b)
        let i32_type = leaked.i32_type();
        let ptr_type = i32_type.ptr_type(inkwell::AddressSpace::default());
        let fn_type = i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let function = module.add_function("test_add", fn_type, None);

        // 创建 entry block
        let entry = leaked.append_basic_block(function, "entry");
        builder.position_at_end(entry);

        let a_ptr = function.get_first_param().unwrap().into_pointer_value();
        let b_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        // 加载参数值
        let a = builder
            .build_load(i32_type, a_ptr, "a")
            .expect("load a failed")
            .into_int_value();
        let b = builder
            .build_load(i32_type, b_ptr, "b")
            .expect("load b failed")
            .into_int_value();

        // 相加并返回
        let sum = builder.build_int_add(a, b, "sum").expect("add failed");
        builder.build_return(Some(&sum)).expect("return failed");

        // 验证 IR
        assert!(function.verify(true), "IR verification failed");

        // 编译并执行
        let fn_ptr: extern "C" fn(*const i32, *const i32) -> i32 = unsafe {
            let f: inkwell::execution_engine::JitFunction<
                unsafe extern "C" fn(*const i32, *const i32) -> i32,
            > = execution_engine
                .get_function("test_add")
                .expect("get_function failed");
            std::mem::transmute(f.as_raw())
        };

        let a_val: i32 = 3;
        let b_val: i32 = 4;
        let result = fn_ptr(&a_val, &b_val);
        assert_eq!(result, 7, "JIT function should return 3 + 4 = 7");
    }

    /// 测试 JIT 编译器初始化
    #[test]
    fn test_jit_init() {
        let result = init();
        assert!(result.is_ok(), "JIT init should succeed: {:?}", result);
    }
}
