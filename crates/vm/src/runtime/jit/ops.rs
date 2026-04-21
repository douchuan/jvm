// ============================================================
// Opcode 翻译函数（运行时调用）
//
// 本模块包含需要调用 JVM 运行时函数的 opcode 翻译实现。
// 例如：new 对象、方法调用、字段访问等——这些不能纯靠 LLVM IR 完成，
// 必须回调到 Rust 运行时。
//
// ## 运行时调用（Runtime Callout）的原理
//
// JIT 编译的代码在 LLVM Module 中声明外部函数（extern function），
// LLVM 在执行时会自动链接到实际的 Rust 函数地址。
//
// 例如，`new` 指令需要创建对象：
// 1. 在 LLVM IR 中声明: declare i32* @runtime_new_inst(i8* class_ptr)
// 2. 调用该函数: %obj = call i32* @runtime_new_inst(%class_ptr)
// 3. LLVM ExecutionEngine 在链接时解析到实际的 Oop::new_inst 函数
//
// MVP 阶段本模块为空——先跑通基本的算术和控制流，后续添加对象操作时再填充。
// ============================================================

// 暂无需要实现的内容，等待后续扩展。
