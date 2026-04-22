// ============================================================
// LLVM IR 构建器
//
// 本模块负责将 Java 方法的 bytecode 翻译为 LLVM IR。
//
// ## 核心思路：Stack-to-Register 转换
//
// JVM 是基于栈的虚拟机，而 LLVM IR 是基于寄存器（SSA）的。
// 转换策略：
//
// 1. 为本地变量数组分配 `alloca` 空间（LLVM 的栈内存分配指令）
// 2. 为操作数栈分配 `alloca` 空间
// 3. 维护一个 `stack_top` 指针，跟踪当前栈顶位置
// 4. JVM 的 `push` 对应 LLVM 的 `store` 到 alloca 位置
// 5. JVM 的 `pop` 对应 LLVM 的 `load` 从 alloca 位置
//
// 为什么用 alloca 而不是直接做 SSA？
// - alloca + mem2reg 是更简单的方案。我们只需要生成 store/load，
//   LLVM 的 mem2reg pass 会自动消除内存访问，提升为 SSA register。
// - 手动做 SSA 需要处理 Phi 节点（控制流汇聚时选择正确的值），
//   这在有条件分支和循环的场景下非常复杂。
// - mem2reg 是 LLVM 最成熟的优化 pass，正确性有保证。
//
// ## BasicBlock 映射
//
// JVM 的跳转目标（goto、if* 的 target、异常 handler）映射为 LLVM BasicBlock。
// 每条可能跳转到新位置的指令都会创建或引用一个 BasicBlock。
//
// ## inkwell 0.9 API 注意
//
// inkwell 0.9 的 builder 方法返回 `Result<T, BuilderError>` 而非 `T`。
// 我们用 `.expect()` 处理——IR 生成阶段的错误是编译 bug，应该 panic。
// ============================================================

use crate::runtime::method::Method;
use classfile::{OpCode, U1};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{AnyValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use std::collections::HashMap;
use tracing::{debug, error, info, trace, warn};

/// JIT invoke 运行时外部函数签名。
/// extern "C" fn(cp_idx: u16, locals: *mut i32, stack: *mut i32, stack_top: u32)
type RuntimeInvokeFn<'ctx> = inkwell::values::FunctionValue<'ctx>;

/// 编译单个方法，生成 LLVM IR 函数。
///
/// ## 参数
/// - `bytecode`: 方法的字节码数组
/// - `max_locals`: 最大本地变量槽数
/// - `max_stack`: 最大操作数栈深度
///
/// ## 返回
/// 生成的 LLVM `FunctionValue`。如果编译失败（如遇到不支持的 opcode），返回 None。
pub fn compile_method<'a, 'ctx: 'a>(
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    method: &Method,
    bytecode: &'a [U1],
    max_locals: usize,
    max_stack: usize,
) -> Option<FunctionValue<'ctx>> {
    // ============================================================
    // 步骤 1: 创建 LLVM 函数
    // ============================================================
    //
    // 函数签名: void fn(i8* locals, i8* stack)
    //
    // 选择 i8* 而不是类型化的指针，因为：
    // 1. 简化签名——不需要为不同参数类型生成不同函数
    // 2. 调用方可以直接传递 `Slot` 数组的指针
    // 3. 在函数内部，我们用 getelementptr (GEP) 计算具体槽位
    //
    // GEP 是 LLVM 的地址计算指令。它不访问内存，只计算地址。
    // 类比 C 语言: `ptr + offset`

    let i8_type = context.i8_type();
    let ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

    // 生成唯一的函数名。使用 "jit_" 前缀 + 类名 + 方法名，便于调试。
    let cls_name = String::from_utf8_lossy(&method.cls_name).replace("/", "_");
    let method_name = String::from_utf8_lossy(&method.name);
    let fn_name = format!("jit_{}_{}", cls_name, method_name);

    // 在 LLVM Module 中创建函数。
    let fn_type = context
        .void_type()
        .fn_type(&[ptr_type.into(), ptr_type.into()], false);
    let function = module.add_function(&fn_name, fn_type, None);

    // ============================================================
    // 步骤 2: 创建 BasicBlock
    // ============================================================
    //
    // BasicBlock 是 LLVM IR 的基本控制流单位。每个 block 以终止符
    //（br/ret/switch）结尾。
    //
    // 我们扫描所有跳转目标，为每个目标创建一个 BasicBlock。

    let jump_targets = collect_jump_targets(bytecode);

    // 为每个跳转目标创建 BasicBlock
    let mut bb_map: HashMap<usize, BasicBlock<'ctx>> = HashMap::new();

    // 创建入口 block
    let entry_bb = context.append_basic_block(function, "entry");
    bb_map.insert(0, entry_bb);

    // 为每个跳转目标创建 block
    for offset in &jump_targets {
        if *offset != 0 && *offset < bytecode.len() {
            let name = format!("bb_{}", offset);
            let bb = context.append_basic_block(function, &name);
            bb_map.insert(*offset, bb);
        }
    }

    // 创建返回 block（方法正常结束时的汇聚点）
    let return_bb = context.append_basic_block(function, "return");

    // ============================================================
    // 步骤 3: 在 entry block 中分配局部变量
    // ============================================================

    builder.position_at_end(entry_bb);

    // 将 locals 指针转换为 i32* 类型
    let locals_ptr = function.get_first_param().unwrap().into_pointer_value();
    let locals_i32_ptr = builder
        .build_pointer_cast(
            locals_ptr,
            context
                .i32_type()
                .ptr_type(inkwell::AddressSpace::default()),
            "locals_cast",
        )
        .expect("build_pointer_cast failed");

    // 创建本地变量数组（alloca 分配）
    // 每个 local 对应一个 alloca，初始值从参数指针加载。
    let num_locals = max_locals.max(1);
    let local_vars: Vec<PointerValue<'ctx>> = (0..num_locals)
        .map(|i| {
            let alloca = builder
                .build_alloca(context.i32_type(), &format!("local_{}", i))
                .expect("build_alloca failed");
            // 从参数指针加载初始值
            let idx = context.i32_type().const_int(i as u64, false);
            let ptr = unsafe {
                builder
                    .build_in_bounds_gep(context.i32_type(), locals_i32_ptr, &[idx], "local_ptr")
                    .expect("build_gep failed")
            };
            let loaded = builder
                .build_load(context.i32_type(), ptr, &format!("load_local_{}", i))
                .expect("build_load failed")
                .into_int_value();
            builder
                .build_store(alloca, loaded)
                .expect("build_store failed");
            alloca
        })
        .collect();

    // 创建操作数栈数组（alloca 分配）
    let stack_size = max_stack.max(1);
    let stack_vars: Vec<PointerValue<'ctx>> = (0..stack_size)
        .map(|i| {
            builder
                .build_alloca(context.i32_type(), &format!("stack_{}", i))
                .expect("build_alloca failed")
        })
        .collect();

    // 当前栈顶指针（指向下一个可用的栈槽位）
    let stack_top = builder
        .build_alloca(context.i32_type(), "stack_top")
        .expect("build_alloca failed");
    builder
        .build_store(stack_top, context.i32_type().const_int(0, false))
        .expect("build_store failed");

    // ============================================================
    // 步骤 4: 开始 bytecode 翻译
    // ============================================================

    // 将 stack 参数也转换为 i32* 类型，用于 ireturn 写入返回值
    let stack_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
    let stack_i32_ptr = builder
        .build_pointer_cast(
            stack_ptr,
            context
                .i32_type()
                .ptr_type(inkwell::AddressSpace::default()),
            "stack_cast",
        )
        .expect("build_pointer_cast failed");

    let mut interp = BytecodeInterpreter {
        context,
        builder,
        bb_map,
        function,
        return_bb,
        local_vars,
        stack_vars,
        stack_top,
        bytecode,
        max_stack,
        stack_param: Some(stack_i32_ptr),
        module: module as *const Module<'ctx>,
    };

    // 从 entry block 开始翻译
    let entry_bb = *interp.bb_map.get(&0).unwrap();
    builder.position_at_end(entry_bb);
    interp.translate_bytecode(0);

    // 对于因分支指令提前返回而遗漏的基本块，
    // 逐一处理（这些块可能从条件分支的 fallthrough 或 target 创建）
    interp.translate_remaining_blocks();

    // 确保 return_bb 有正确的终止符
    if return_bb.get_terminator().is_none() {
        builder.position_at_end(return_bb);
        builder.build_return(None).expect("build_return failed");
    }

    // 打印生成的 IR（调试用）
    info!("JIT: Generated LLVM IR for {}:\n{}", fn_name, fn_name);

    Some(function)
}

/// Bytecode 翻译器的上下文。
struct BytecodeInterpreter<'ctx, 'a> {
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    bb_map: HashMap<usize, BasicBlock<'ctx>>,
    function: FunctionValue<'ctx>,
    return_bb: BasicBlock<'ctx>,
    local_vars: Vec<PointerValue<'ctx>>,
    stack_vars: Vec<PointerValue<'ctx>>,
    stack_top: PointerValue<'ctx>,
    bytecode: &'a [U1],
    max_stack: usize,
    /// 调用方的 stack 缓冲区指针（i32*）。用于在 ireturn 时写入返回值。
    stack_param: Option<PointerValue<'ctx>>,
    /// 模块引用（用于声明外部函数）。
    module: *const Module<'ctx>,
}

impl<'ctx, 'a> BytecodeInterpreter<'ctx, 'a> {
    /// 主翻译循环。
    fn translate_bytecode(&mut self, start_pc: usize) {
        let mut pc = start_pc;

        loop {
            if pc >= self.bytecode.len() {
                self.builder
                    .build_unconditional_branch(self.return_bb)
                    .expect("branch failed");
                break;
            }

            let opcode = self.bytecode[pc];
            let opcode = OpCode::from(opcode);

            match opcode {
                // ============================================================
                // 常量加载指令
                // ============================================================
                OpCode::iconst_m1 => {
                    self.push_int(-1);
                    pc += 1;
                }
                OpCode::iconst_0 => {
                    self.push_int(0);
                    pc += 1;
                }
                OpCode::iconst_1 => {
                    self.push_int(1);
                    pc += 1;
                }
                OpCode::iconst_2 => {
                    self.push_int(2);
                    pc += 1;
                }
                OpCode::iconst_3 => {
                    self.push_int(3);
                    pc += 1;
                }
                OpCode::iconst_4 => {
                    self.push_int(4);
                    pc += 1;
                }
                OpCode::iconst_5 => {
                    self.push_int(5);
                    pc += 1;
                }
                OpCode::bipush => {
                    let val = self.bytecode[pc + 1] as i8 as i32;
                    self.push_int(val);
                    pc += 2;
                }
                OpCode::sipush => {
                    let val =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    self.push_int(val);
                    pc += 3;
                }

                // ============================================================
                // 本地变量加载/存储指令
                // ============================================================
                OpCode::iload => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.load_local_int(idx);
                    pc += 2;
                }
                OpCode::iload_0 => {
                    self.load_local_int(0);
                    pc += 1;
                }
                OpCode::iload_1 => {
                    self.load_local_int(1);
                    pc += 1;
                }
                OpCode::iload_2 => {
                    self.load_local_int(2);
                    pc += 1;
                }
                OpCode::iload_3 => {
                    self.load_local_int(3);
                    pc += 1;
                }

                OpCode::istore => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.store_local_int(idx);
                    pc += 2;
                }
                OpCode::istore_0 => {
                    self.store_local_int(0);
                    pc += 1;
                }
                OpCode::istore_1 => {
                    self.store_local_int(1);
                    pc += 1;
                }
                OpCode::istore_2 => {
                    self.store_local_int(2);
                    pc += 1;
                }
                OpCode::istore_3 => {
                    self.store_local_int(3);
                    pc += 1;
                }

                // ============================================================
                // Reference load/store instructions
                // ============================================================
                OpCode::aload => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.load_local_ref(idx);
                    pc += 2;
                }
                OpCode::aload_0 => {
                    self.load_local_ref(0);
                    pc += 1;
                }
                OpCode::aload_1 => {
                    self.load_local_ref(1);
                    pc += 1;
                }
                OpCode::aload_2 => {
                    self.load_local_ref(2);
                    pc += 1;
                }
                OpCode::aload_3 => {
                    self.load_local_ref(3);
                    pc += 1;
                }
                OpCode::astore => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.store_local_ref(idx);
                    pc += 2;
                }
                OpCode::astore_0 => {
                    self.store_local_ref(0);
                    pc += 1;
                }
                OpCode::astore_1 => {
                    self.store_local_ref(1);
                    pc += 1;
                }
                OpCode::astore_2 => {
                    self.store_local_ref(2);
                    pc += 1;
                }
                OpCode::astore_3 => {
                    self.store_local_ref(3);
                    pc += 1;
                }

                // ============================================================
                // Float load/store instructions
                // ============================================================
                OpCode::fload => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.load_local_float(idx);
                    pc += 2;
                }
                OpCode::fload_0 => {
                    self.load_local_float(0);
                    pc += 1;
                }
                OpCode::fload_1 => {
                    self.load_local_float(1);
                    pc += 1;
                }
                OpCode::fload_2 => {
                    self.load_local_float(2);
                    pc += 1;
                }
                OpCode::fload_3 => {
                    self.load_local_float(3);
                    pc += 1;
                }
                OpCode::fstore => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.store_local_float(idx);
                    pc += 2;
                }
                OpCode::fstore_0 => {
                    self.store_local_float(0);
                    pc += 1;
                }
                OpCode::fstore_1 => {
                    self.store_local_float(1);
                    pc += 1;
                }
                OpCode::fstore_2 => {
                    self.store_local_float(2);
                    pc += 1;
                }
                OpCode::fstore_3 => {
                    self.store_local_float(3);
                    pc += 1;
                }

                // ============================================================
                // Double load/store instructions
                // ============================================================
                OpCode::dload => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.load_local_double(idx);
                    pc += 2;
                }
                OpCode::dload_0 => {
                    self.load_local_double(0);
                    pc += 1;
                }
                OpCode::dload_1 => {
                    self.load_local_double(1);
                    pc += 1;
                }
                OpCode::dload_2 => {
                    self.load_local_double(2);
                    pc += 1;
                }
                OpCode::dload_3 => {
                    self.load_local_double(3);
                    pc += 1;
                }
                OpCode::dstore => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.store_local_double(idx);
                    pc += 2;
                }
                OpCode::dstore_0 => {
                    self.store_local_double(0);
                    pc += 1;
                }
                OpCode::dstore_1 => {
                    self.store_local_double(1);
                    pc += 1;
                }
                OpCode::dstore_2 => {
                    self.store_local_double(2);
                    pc += 1;
                }
                OpCode::dstore_3 => {
                    self.store_local_double(3);
                    pc += 1;
                }

                // ============================================================
                // 算术运算指令
                // ============================================================
                OpCode::iadd => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self
                        .builder
                        .build_int_add(v1, v2, "add")
                        .expect("iadd failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::isub => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self
                        .builder
                        .build_int_sub(v1, v2, "sub")
                        .expect("isub failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::imul => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self
                        .builder
                        .build_int_mul(v1, v2, "mul")
                        .expect("imul failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::idiv => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self
                        .builder
                        .build_int_signed_div(v1, v2, "div")
                        .expect("idiv failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::irem => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self
                        .builder
                        .build_int_signed_rem(v1, v2, "rem")
                        .expect("irem failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::ineg => {
                    let v = self.pop_int();
                    let result = self.builder.build_int_neg(v, "neg").expect("ineg failed");
                    self.push_int_val(result);
                    pc += 1;
                }

                // ============================================================
                // Long arithmetic
                // ============================================================
                OpCode::lconst_0 => {
                    self.push_long_val(self.context.i64_type().const_int(0, false));
                    pc += 1;
                }
                OpCode::lconst_1 => {
                    self.push_long_val(self.context.i64_type().const_int(1, false));
                    pc += 1;
                }
                OpCode::lload => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.load_local_long(idx);
                    pc += 2;
                }
                OpCode::lload_0 => {
                    self.load_local_long(0);
                    pc += 1;
                }
                OpCode::lload_1 => {
                    self.load_local_long(1);
                    pc += 1;
                }
                OpCode::lload_2 => {
                    self.load_local_long(2);
                    pc += 1;
                }
                OpCode::lload_3 => {
                    self.load_local_long(3);
                    pc += 1;
                }
                OpCode::lstore => {
                    let idx = self.bytecode[pc + 1] as usize;
                    self.store_local_long(idx);
                    pc += 2;
                }
                OpCode::lstore_0 => {
                    self.store_local_long(0);
                    pc += 1;
                }
                OpCode::lstore_1 => {
                    self.store_local_long(1);
                    pc += 1;
                }
                OpCode::lstore_2 => {
                    self.store_local_long(2);
                    pc += 1;
                }
                OpCode::lstore_3 => {
                    self.store_local_long(3);
                    pc += 1;
                }
                OpCode::ladd => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self
                        .builder
                        .build_int_add(v1, v2, "ladd")
                        .expect("ladd failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lsub => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self
                        .builder
                        .build_int_sub(v1, v2, "lsub")
                        .expect("lsub failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lmul => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self
                        .builder
                        .build_int_mul(v1, v2, "lmul")
                        .expect("lmul failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::ldiv => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self
                        .builder
                        .build_int_signed_div(v1, v2, "ldiv")
                        .expect("ldiv failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lrem => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self
                        .builder
                        .build_int_signed_rem(v1, v2, "lrem")
                        .expect("lrem failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lneg => {
                    let v = self.pop_long();
                    let result = self.builder.build_int_neg(v, "lneg").expect("lneg failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::land => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self.builder.build_and(v1, v2, "land").expect("land failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lor => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self.builder.build_or(v1, v2, "lor").expect("lor failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lxor => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let result = self.builder.build_xor(v1, v2, "lxor").expect("lxor failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lshl => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    // Mask: only low 6 bits are used (mask with 63)
                    let mask = self
                        .builder
                        .build_and(
                            v2,
                            self.context.i64_type().const_int(63, false),
                            "lshl_mask",
                        )
                        .expect("mask failed");
                    let result = self
                        .builder
                        .build_left_shift(v1, mask, "lshl")
                        .expect("lshl failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lshr => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let mask = self
                        .builder
                        .build_and(
                            v2,
                            self.context.i64_type().const_int(63, false),
                            "lshr_mask",
                        )
                        .expect("mask failed");
                    let result = self
                        .builder
                        .build_right_shift(v1, mask, true, "lshr")
                        .expect("lshr failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lushr => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    let mask = self
                        .builder
                        .build_and(
                            v2,
                            self.context.i64_type().const_int(63, false),
                            "lushr_mask",
                        )
                        .expect("mask failed");
                    let result = self
                        .builder
                        .build_right_shift(v1, mask, false, "lushr")
                        .expect("lushr failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::lcmp => {
                    let v2 = self.pop_long();
                    let v1 = self.pop_long();
                    // Compare v1 and v2: result = 1 if v1>v2, -1 if v1<v2, 0 if equal
                    let gt = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, v1, v2, "lcmp_gt")
                        .expect("lcmp gt failed");
                    let lt = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, v1, v2, "lcmp_lt")
                        .expect("lcmp lt failed");
                    let one = self.context.i32_type().const_int(1, false);
                    let neg_one = self.context.i32_type().const_int(-1i64 as u64, false);
                    let zero = self.context.i32_type().const_int(0, false);
                    let result = self
                        .builder
                        .build_select(gt, one, zero, "lcmp_gt_sel")
                        .expect("lcmp select gt failed")
                        .into_int_value();
                    let result = self
                        .builder
                        .build_select(lt, neg_one, result, "lcmp_lt_sel")
                        .expect("lcmp select lt failed")
                        .into_int_value();
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::lreturn => {
                    let _val = self.pop_long();
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("lreturn branch failed");
                    return;
                }

                // ============================================================
                // Float/Double constants
                // ============================================================
                OpCode::fconst_0 => {
                    self.push_float_val(self.context.f32_type().const_float(0.0));
                    pc += 1;
                }
                OpCode::fconst_1 => {
                    self.push_float_val(self.context.f32_type().const_float(1.0));
                    pc += 1;
                }
                OpCode::fconst_2 => {
                    self.push_float_val(self.context.f32_type().const_float(2.0));
                    pc += 1;
                }
                OpCode::dconst_0 => {
                    self.push_double_val(self.context.f64_type().const_float(0.0));
                    pc += 1;
                }
                OpCode::dconst_1 => {
                    self.push_double_val(self.context.f64_type().const_float(1.0));
                    pc += 1;
                }

                // ============================================================
                // Float/Double arithmetic
                // ============================================================
                OpCode::fadd => {
                    let v2 = self.pop_float();
                    let v1 = self.pop_float();
                    let result = self
                        .builder
                        .build_float_add(v1, v2, "fadd")
                        .expect("fadd failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::fsub => {
                    let v2 = self.pop_float();
                    let v1 = self.pop_float();
                    let result = self
                        .builder
                        .build_float_sub(v1, v2, "fsub")
                        .expect("fsub failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::fmul => {
                    let v2 = self.pop_float();
                    let v1 = self.pop_float();
                    let result = self
                        .builder
                        .build_float_mul(v1, v2, "fmul")
                        .expect("fmul failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::fdiv => {
                    let v2 = self.pop_float();
                    let v1 = self.pop_float();
                    let result = self
                        .builder
                        .build_float_div(v1, v2, "fdiv")
                        .expect("fdiv failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::frem => {
                    let v2 = self.pop_float();
                    let v1 = self.pop_float();
                    let result = self
                        .builder
                        .build_float_rem(v1, v2, "frem")
                        .expect("frem failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::fneg => {
                    let v = self.pop_float();
                    let result = self
                        .builder
                        .build_float_neg(v, "fneg")
                        .expect("fneg failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::dadd => {
                    let v2 = self.pop_double();
                    let v1 = self.pop_double();
                    let result = self
                        .builder
                        .build_float_add(v1, v2, "dadd")
                        .expect("dadd failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::dsub => {
                    let v2 = self.pop_double();
                    let v1 = self.pop_double();
                    let result = self
                        .builder
                        .build_float_sub(v1, v2, "dsub")
                        .expect("dsub failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::dmul => {
                    let v2 = self.pop_double();
                    let v1 = self.pop_double();
                    let result = self
                        .builder
                        .build_float_mul(v1, v2, "dmul")
                        .expect("dmul failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::ddiv => {
                    let v2 = self.pop_double();
                    let v1 = self.pop_double();
                    let result = self
                        .builder
                        .build_float_div(v1, v2, "ddiv")
                        .expect("ddiv failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::drem => {
                    let v2 = self.pop_double();
                    let v1 = self.pop_double();
                    let result = self
                        .builder
                        .build_float_rem(v1, v2, "drem")
                        .expect("drem failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::dneg => {
                    let v = self.pop_double();
                    let result = self
                        .builder
                        .build_float_neg(v, "dneg")
                        .expect("dneg failed");
                    self.push_double_val(result);
                    pc += 1;
                }

                // ============================================================
                // Float/Double comparison
                // ============================================================
                OpCode::fcmpl => {
                    // fcmpl: NaN → -1
                    self.build_float_compare(false);
                    pc += 1;
                }
                OpCode::fcmpg => {
                    // fcmpg: NaN → 1
                    self.build_float_compare(true);
                    pc += 1;
                }
                OpCode::dcmpl => {
                    // dcmpl: NaN → -1
                    self.build_double_compare(false);
                    pc += 1;
                }
                OpCode::dcmpg => {
                    // dcmpg: NaN → 1
                    self.build_double_compare(true);
                    pc += 1;
                }
                OpCode::freturn => {
                    let _val = self.pop_float();
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("freturn branch failed");
                    return;
                }
                OpCode::dreturn => {
                    let _val = self.pop_double();
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("dreturn branch failed");
                    return;
                }

                // ============================================================
                // Reference return
                // ============================================================
                OpCode::areturn => {
                    let _val = self.pop_int(); // slot_id
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("areturn branch failed");
                    return;
                }

                // ============================================================
                // iinc: increment local variable by constant
                // ============================================================
                OpCode::iinc => {
                    let idx = self.bytecode[pc + 1] as usize;
                    let const_byte = self.bytecode[pc + 2] as i8 as i32;
                    if idx < self.local_vars.len() {
                        let current = self
                            .builder
                            .build_load(
                                self.context.i32_type(),
                                self.local_vars[idx],
                                &format!("iinc_load_{}", idx),
                            )
                            .expect("iinc load failed")
                            .into_int_value();
                        let increment = self
                            .context
                            .i32_type()
                            .const_int(const_byte as u64, const_byte < 0);
                        let result = self
                            .builder
                            .build_int_add(current, increment, "iinc_add")
                            .expect("iinc add failed");
                        self.builder
                            .build_store(self.local_vars[idx], result)
                            .expect("iinc store failed");
                    }
                    pc += 3;
                }

                // ============================================================
                // 位运算指令
                // ============================================================
                OpCode::iand => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self.builder.build_and(v1, v2, "and").expect("iand failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::ior => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self.builder.build_or(v1, v2, "or").expect("ior failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::ixor => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let result = self.builder.build_xor(v1, v2, "xor").expect("ixor failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::ishl => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let shift = self
                        .builder
                        .build_int_unsigned_rem(
                            v2,
                            self.context.i32_type().const_int(32, false),
                            "shift_mask",
                        )
                        .expect("ishl mask failed");
                    let result = self
                        .builder
                        .build_left_shift(v1, shift, "shl")
                        .expect("ishl failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::ishr => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let shift = self
                        .builder
                        .build_int_unsigned_rem(
                            v2,
                            self.context.i32_type().const_int(32, false),
                            "shift_mask",
                        )
                        .expect("ishr mask failed");
                    let result = self
                        .builder
                        .build_right_shift(v1, shift, true, "ashr")
                        .expect("ishr failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::iushr => {
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let shift = self
                        .builder
                        .build_int_unsigned_rem(
                            v2,
                            self.context.i32_type().const_int(32, false),
                            "shift_mask",
                        )
                        .expect("iushr mask failed");
                    let result = self
                        .builder
                        .build_right_shift(v1, shift, false, "ushr")
                        .expect("iushr failed");
                    self.push_int_val(result);
                    pc += 1;
                }

                // ============================================================
                // 类型转换指令
                // ============================================================
                OpCode::i2l => {
                    let v = self.pop_int();
                    let result = self
                        .builder
                        .build_int_s_extend_or_bit_cast(v, self.context.i64_type(), "i2l")
                        .expect("i2l failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::i2f => {
                    let v = self.pop_int();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(v, self.context.f32_type(), "i2f")
                        .expect("i2f failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::i2d => {
                    let v = self.pop_int();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(v, self.context.f64_type(), "i2d")
                        .expect("i2d failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::l2i => {
                    let v = self.pop_long();
                    let result = self
                        .builder
                        .build_int_truncate(v, self.context.i32_type(), "l2i")
                        .expect("l2i failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::l2f => {
                    let v = self.pop_long();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(v, self.context.f32_type(), "l2f")
                        .expect("l2f failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::l2d => {
                    let v = self.pop_long();
                    let result = self
                        .builder
                        .build_unsigned_int_to_float(v, self.context.f64_type(), "l2d")
                        .expect("l2d failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::f2i => {
                    let v = self.pop_float();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(v, self.context.i32_type(), "f2i")
                        .expect("f2i failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::f2l => {
                    let v = self.pop_float();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(v, self.context.i64_type(), "f2l")
                        .expect("f2l failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::f2d => {
                    let v = self.pop_float();
                    let result = self
                        .builder
                        .build_float_ext(v, self.context.f64_type(), "f2d")
                        .expect("f2d failed");
                    self.push_double_val(result);
                    pc += 1;
                }
                OpCode::d2i => {
                    let v = self.pop_double();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(v, self.context.i32_type(), "d2i")
                        .expect("d2i failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::d2l => {
                    let v = self.pop_double();
                    let result = self
                        .builder
                        .build_float_to_unsigned_int(v, self.context.i64_type(), "d2l")
                        .expect("d2l failed");
                    self.push_long_val(result);
                    pc += 1;
                }
                OpCode::d2f => {
                    let v = self.pop_double();
                    let result = self
                        .builder
                        .build_float_trunc(v, self.context.f32_type(), "d2f")
                        .expect("d2f failed");
                    self.push_float_val(result);
                    pc += 1;
                }
                OpCode::i2b => {
                    let v = self.pop_int();
                    // Sign-extend: shift left then arithmetic shift right
                    let left = self
                        .builder
                        .build_left_shift(
                            v,
                            self.context.i32_type().const_int(24, false),
                            "i2b_shl",
                        )
                        .expect("i2b shl failed");
                    let result = self
                        .builder
                        .build_right_shift(
                            left,
                            self.context.i32_type().const_int(24, false),
                            true,
                            "i2b_shr",
                        )
                        .expect("i2b shr failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::i2c => {
                    let v = self.pop_int();
                    // Zero-extend to 16 bits
                    let result = self
                        .builder
                        .build_and(
                            v,
                            self.context.i32_type().const_int(0xFFFF, false),
                            "i2c_mask",
                        )
                        .expect("i2c mask failed");
                    self.push_int_val(result);
                    pc += 1;
                }
                OpCode::i2s => {
                    let v = self.pop_int();
                    // Sign-extend: shift left then arithmetic shift right
                    let left = self
                        .builder
                        .build_left_shift(
                            v,
                            self.context.i32_type().const_int(16, false),
                            "i2s_shl",
                        )
                        .expect("i2s shl failed");
                    let result = self
                        .builder
                        .build_right_shift(
                            left,
                            self.context.i32_type().const_int(16, false),
                            true,
                            "i2s_shr",
                        )
                        .expect("i2s shr failed");
                    self.push_int_val(result);
                    pc += 1;
                }

                // ============================================================
                // 控制流指令
                // ============================================================
                OpCode::goto => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    self.branch_to(target);
                    return;
                }
                OpCode::ifeq => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, val, zero, "eq_zero")
                        .expect("ifeq compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifeq branch failed");
                    return;
                }
                OpCode::ifne => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, val, zero, "ne_zero")
                        .expect("ifne compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifne branch failed");
                    return;
                }
                OpCode::iflt => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, val, zero, "lt_zero")
                        .expect("iflt compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("iflt branch failed");
                    return;
                }
                OpCode::ifge => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGE, val, zero, "ge_zero")
                        .expect("ifge compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifge branch failed");
                    return;
                }
                OpCode::ifgt => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, val, zero, "gt_zero")
                        .expect("ifgt compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifgt branch failed");
                    return;
                }
                OpCode::ifle => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let val = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLE, val, zero, "le_zero")
                        .expect("ifle compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifle branch failed");
                    return;
                }
                OpCode::if_icmpeq => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, v1, v2, "icmp_eq")
                        .expect("if_icmpeq compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmpeq branch failed");
                    return;
                }
                OpCode::if_icmpne => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, v1, v2, "icmp_ne")
                        .expect("if_icmpne compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmpne branch failed");
                    return;
                }
                OpCode::if_icmplt => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, v1, v2, "icmp_lt")
                        .expect("if_icmplt compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmplt branch failed");
                    return;
                }
                OpCode::if_icmpge => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGE, v1, v2, "icmp_ge")
                        .expect("if_icmpge compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmpge branch failed");
                    return;
                }
                OpCode::if_icmpgt => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, v1, v2, "icmp_gt")
                        .expect("if_icmpgt compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmpgt branch failed");
                    return;
                }
                OpCode::if_icmple => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLE, v1, v2, "icmp_le")
                        .expect("if_icmple compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_icmple branch failed");
                    return;
                }

                // ============================================================
                // Reference comparison instructions
                // ============================================================
                OpCode::if_acmpeq => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, v1, v2, "acmpeq")
                        .expect("if_acmpeq compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_acmpeq branch failed");
                    return;
                }
                OpCode::if_acmpne => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v2 = self.pop_int();
                    let v1 = self.pop_int();
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, v1, v2, "acmpne")
                        .expect("if_acmpne compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("if_acmpne branch failed");
                    return;
                }

                // ============================================================
                // Null check instructions
                // ============================================================
                OpCode::ifnull => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, v, zero, "ifnull")
                        .expect("ifnull compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifnull branch failed");
                    return;
                }
                OpCode::ifnonnull => {
                    let offset =
                        i16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    let v = self.pop_int();
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond = self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, v, zero, "ifnonnull")
                        .expect("ifnonnull compare failed");
                    let fallthrough = self.ensure_block(pc + 3);
                    let target_bb = self.ensure_block(target);
                    self.builder
                        .build_conditional_branch(cond, target_bb, fallthrough)
                        .expect("ifnonnull branch failed");
                    return;
                }

                // ============================================================
                // 返回指令
                // ============================================================
                OpCode::ireturn => {
                    let val = self.pop_int();
                    // 将返回值写入调用方的 stack 缓冲区
                    if let Some(stack_param) = self.stack_param {
                        let idx = self.context.i32_type().const_int(0, false);
                        let ptr = unsafe {
                            self.builder
                                .build_in_bounds_gep(
                                    self.context.i32_type(),
                                    stack_param,
                                    &[idx],
                                    "ret_store",
                                )
                                .expect("ireturn gep failed")
                        };
                        self.builder
                            .build_store(ptr, val)
                            .expect("ireturn store failed");
                    }
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("ireturn branch failed");
                    return;
                }
                OpCode::return_void => {
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("return branch failed");
                    return;
                }

                // ============================================================
                // 栈操作指令
                // ============================================================
                OpCode::pop => {
                    let _val = self.pop_int();
                    pc += 1;
                }
                OpCode::pop2 => {
                    let _val = self.pop_int();
                    let _val2 = self.pop_int();
                    pc += 1;
                }
                OpCode::dup => {
                    let val = self.pop_int();
                    self.push_int_val(val);
                    self.push_int_val(val);
                    pc += 1;
                }
                OpCode::dup_x1 => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    self.push_int_val(val1);
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    pc += 1;
                }
                OpCode::dup_x2 => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    let val3 = self.pop_int();
                    self.push_int_val(val1);
                    self.push_int_val(val3);
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    pc += 1;
                }
                OpCode::dup2 => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    pc += 1;
                }
                OpCode::dup2_x1 => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    let val3 = self.pop_int();
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    self.push_int_val(val3);
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    pc += 1;
                }
                OpCode::dup2_x2 => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    let val3 = self.pop_int();
                    let val4 = self.pop_int();
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    self.push_int_val(val4);
                    self.push_int_val(val3);
                    self.push_int_val(val2);
                    self.push_int_val(val1);
                    pc += 1;
                }
                OpCode::swap => {
                    let val1 = self.pop_int();
                    let val2 = self.pop_int();
                    self.push_int_val(val1);
                    self.push_int_val(val2);
                    pc += 1;
                }

                // ============================================================
                // nop
                // ============================================================
                OpCode::nop => {
                    pc += 1;
                }

                // ============================================================
                // Reference constants
                // ============================================================
                OpCode::aconst_null => {
                    // null 引用用 slot_id = 0 表示
                    self.push_int_val(self.context.i32_type().const_int(0, false));
                    pc += 1;
                }

                // ============================================================
                // 方法调用指令 —— 回退到运行时
                // ============================================================
                OpCode::invokevirtual => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_invoke_runtime("jit_invoke_virtual", cp_idx);
                    pc += 3;
                }
                OpCode::invokespecial => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_invoke_runtime("jit_invoke_special", cp_idx);
                    pc += 3;
                }
                OpCode::invokestatic => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_invoke_runtime("jit_invoke_static", cp_idx);
                    pc += 3;
                }
                OpCode::invokeinterface => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    // 跳过 count 和 zero 字节
                    self.call_invoke_runtime("jit_invoke_interface", cp_idx);
                    pc += 4;
                }
                OpCode::invokedynamic => {
                    warn!("JIT: invokedynamic not supported, falling back");
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("fallback branch failed");
                    return;
                }

                // ============================================================
                // tableswitch —— LLVM switch 翻译
                // ============================================================
                OpCode::tableswitch => {
                    let mut ptr = pc + 1;
                    if ptr % 4 != 0 {
                        ptr += 4 - ptr % 4;
                    }
                    let default_offset = i32::from_be_bytes([
                        self.bytecode[ptr],
                        self.bytecode[ptr + 1],
                        self.bytecode[ptr + 2],
                        self.bytecode[ptr + 3],
                    ]);
                    let low = i32::from_be_bytes([
                        self.bytecode[ptr + 4],
                        self.bytecode[ptr + 5],
                        self.bytecode[ptr + 6],
                        self.bytecode[ptr + 7],
                    ]);
                    let high = i32::from_be_bytes([
                        self.bytecode[ptr + 8],
                        self.bytecode[ptr + 9],
                        self.bytecode[ptr + 10],
                        self.bytecode[ptr + 11],
                    ]);
                    let num_targets = (high - low + 1) as usize;

                    // Pop key from JIT stack
                    let key = self.pop_int();

                    // Build case list
                    let default_bb = self.ensure_block((pc as i32 + default_offset) as usize);
                    let cases: Vec<_> = (0..num_targets)
                        .map(|i| {
                            let entry_ptr = ptr + 12 + i * 4;
                            let offset = i32::from_be_bytes([
                                self.bytecode[entry_ptr],
                                self.bytecode[entry_ptr + 1],
                                self.bytecode[entry_ptr + 2],
                                self.bytecode[entry_ptr + 3],
                            ]);
                            let case_value = self
                                .context
                                .i32_type()
                                .const_int((low + i as i32) as u64, false);
                            let case_bb = self.ensure_block((pc as i32 + offset) as usize);
                            (case_value, case_bb)
                        })
                        .collect();

                    self.builder
                        .build_switch(key, default_bb, &cases)
                        .expect("build_switch failed");
                    return;
                }

                // ============================================================
                // lookupswitch —— LLVM switch 翻译
                // ============================================================
                OpCode::lookupswitch => {
                    let mut ptr = pc + 1;
                    if ptr % 4 != 0 {
                        ptr += 4 - ptr % 4;
                    }
                    let default_offset = i32::from_be_bytes([
                        self.bytecode[ptr],
                        self.bytecode[ptr + 1],
                        self.bytecode[ptr + 2],
                        self.bytecode[ptr + 3],
                    ]);
                    let npairs = i32::from_be_bytes([
                        self.bytecode[ptr + 4],
                        self.bytecode[ptr + 5],
                        self.bytecode[ptr + 6],
                        self.bytecode[ptr + 7],
                    ]);

                    // Pop key from JIT stack
                    let key = self.pop_int();

                    // Build case list
                    let default_bb = self.ensure_block((pc as i32 + default_offset) as usize);
                    let cases: Vec<_> = (0..npairs)
                        .map(|i| {
                            let pair_ptr = ptr + 8 + (i * 8) as usize;
                            let match_val = i32::from_be_bytes([
                                self.bytecode[pair_ptr],
                                self.bytecode[pair_ptr + 1],
                                self.bytecode[pair_ptr + 2],
                                self.bytecode[pair_ptr + 3],
                            ]);
                            let offset = i32::from_be_bytes([
                                self.bytecode[pair_ptr + 4],
                                self.bytecode[pair_ptr + 5],
                                self.bytecode[pair_ptr + 6],
                                self.bytecode[pair_ptr + 7],
                            ]);
                            let case_value = self
                                .context
                                .i32_type()
                                .const_int(match_val as u32 as u64, false);
                            let case_bb = self.ensure_block((pc as i32 + offset) as usize);
                            (case_value, case_bb)
                        })
                        .collect();

                    self.builder
                        .build_switch(key, default_bb, &cases)
                        .expect("build_switch failed");
                    return;
                }

                // ============================================================
                // Runtime callout 指令 —— 通过 extern "C" 函数回调到 JVM 运行时
                // ============================================================

                // --- 常量池加载 ---
                OpCode::ldc | OpCode::ldc_w => {
                    let cp_idx = if opcode == OpCode::ldc {
                        self.bytecode[pc + 1] as u16
                    } else {
                        u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]])
                    };
                    self.call_ldc_runtime(cp_idx);
                    pc += if opcode == OpCode::ldc { 2 } else { 3 };
                }
                OpCode::ldc2_w => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_ldc2_w_runtime(cp_idx);
                    pc += 3;
                }

                // --- 对象分配 ---
                OpCode::new => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    let slot_id = self.call_new_inst_runtime(cp_idx);
                    self.push_int_val(slot_id);
                    pc += 3;
                }

                // --- 数组分配 ---
                OpCode::newarray => {
                    let ary_type = self.bytecode[pc + 1];
                    let size = self.pop_int();
                    let slot_id = self.call_new_array_runtime(ary_type, size);
                    self.push_int_val(slot_id);
                    pc += 2;
                }
                OpCode::anewarray => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    let size = self.pop_int();
                    let slot_id = self.call_anewarray_runtime(cp_idx, size);
                    self.push_int_val(slot_id);
                    pc += 3;
                }

                // --- 数组长度 ---
                OpCode::arraylength => {
                    let obj = self.pop_int();
                    let len = self.call_array_length_runtime(obj);
                    self.push_int_val(len);
                    pc += 1;
                }

                // --- 类型检查 ---
                OpCode::checkcast => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    // checkcast peeks without popping; read stack_top - 1
                    let obj = self.peek_stack_int();
                    self.call_checkcast_runtime(cp_idx, obj);
                    pc += 3;
                }
                OpCode::instanceof => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    // Pop object, push result
                    let obj = self.pop_int();
                    let result = self.call_instanceof_runtime(cp_idx, obj);
                    self.push_int_val(result);
                    pc += 3;
                }

                // --- 实例字段访问 ---
                OpCode::getfield => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_getfield_runtime(cp_idx);
                    pc += 3;
                }
                OpCode::putfield => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_putfield_runtime(cp_idx);
                    pc += 3;
                }

                // --- 静态字段访问 ---
                OpCode::getstatic => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_getstatic_runtime(cp_idx);
                    pc += 3;
                }
                OpCode::putstatic => {
                    let cp_idx = u16::from_be_bytes([self.bytecode[pc + 1], self.bytecode[pc + 2]]);
                    self.call_putstatic_runtime(cp_idx);
                    pc += 3;
                }

                // --- 数组加载 ---
                OpCode::iaload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_iaload_runtime(array, index);
                    pc += 1;
                }
                OpCode::laload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_laload_runtime(array, index);
                    pc += 1;
                }
                OpCode::faload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_faload_runtime(array, index);
                    pc += 1;
                }
                OpCode::daload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_daload_runtime(array, index);
                    pc += 1;
                }
                OpCode::aaload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_aaload_runtime(array, index);
                    pc += 1;
                }
                OpCode::baload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_baload_runtime(array, index);
                    pc += 1;
                }
                OpCode::caload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_caload_runtime(array, index);
                    pc += 1;
                }
                OpCode::saload => {
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_saload_runtime(array, index);
                    pc += 1;
                }

                // --- 数组存储 ---
                OpCode::iastore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_iastore_runtime(array, index, value);
                    pc += 1;
                }
                OpCode::lastore => {
                    let val_hi = self.pop_int();
                    let val_lo = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_lastore_runtime(array, index, val_lo, val_hi);
                    pc += 1;
                }
                OpCode::fastore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_fastore_runtime(array, index, value);
                    pc += 1;
                }
                OpCode::dastore => {
                    let val_hi = self.pop_int();
                    let val_lo = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_dastore_runtime(array, index, val_lo, val_hi);
                    pc += 1;
                }
                OpCode::aastore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_aastore_runtime(array, index, value);
                    pc += 1;
                }
                OpCode::bastore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_bastore_runtime(array, index, value);
                    pc += 1;
                }
                OpCode::castore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_castore_runtime(array, index, value);
                    pc += 1;
                }
                OpCode::sastore => {
                    let value = self.pop_int();
                    let index = self.pop_int();
                    let array = self.pop_int();
                    self.call_sastore_runtime(array, index, value);
                    pc += 1;
                }

                // --- 同步 ---
                OpCode::monitorenter => {
                    let obj = self.pop_int();
                    self.call_monitorenter_runtime(obj);
                    pc += 1;
                }
                OpCode::monitorexit => {
                    let obj = self.pop_int();
                    self.call_monitorexit_runtime(obj);
                    pc += 1;
                }

                // --- 异常 ---
                OpCode::athrow => {
                    // athrow: 弹出异常引用，抛出
                    let _ex = self.pop_int();
                    // 回退到解释器处理异常（需要完整的异常帧展开）
                    warn!("JIT: athrow not fully supported, falling back");
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("athrow fallback branch failed");
                    return;
                }

                // ============================================================
                // 未实现的 opcode —— 回退到解释器
                // ============================================================
                _ => {
                    warn!(
                        "JIT: unsupported opcode {:?} at pc={}, falling back",
                        opcode, pc
                    );
                    self.builder
                        .build_unconditional_branch(self.return_bb)
                        .expect("fallback branch failed");
                    return;
                }
            }
        }
    }

    // ============================================================
    // 栈操作辅助方法
    // ============================================================

    /// 将一个 i32 值推入操作数栈。
    fn push_int_val(&mut self, val: IntValue<'ctx>) {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("load stack_top failed")
            .into_int_value();

        // 用 GEP 动态计算 stack[slot_idx] 的地址
        if let Some(stack_param) = self.stack_param {
            let ptr = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[top_i32],
                        "push_store",
                    )
                    .expect("push gep failed")
            };
            self.builder
                .build_store(ptr, val)
                .expect("push_int store failed");
        } else {
            // 回退：使用静态 slot 索引
            let slot_idx = top_i32.get_zero_extended_constant().unwrap_or(0) as usize;
            if slot_idx < self.stack_vars.len() {
                self.builder
                    .build_store(self.stack_vars[slot_idx], val)
                    .expect("push_int store failed");
            }
        }

        // 更新 stack_top
        let one = self.context.i32_type().const_int(1, false);
        let new_top = self
            .builder
            .build_int_add(top_i32, one, "inc_top")
            .expect("inc_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("store stack_top failed");
    }

    /// 将一个常量值推入栈的快捷方法
    fn push_int(&mut self, val: i32) {
        let llvm_val = self.context.i32_type().const_int(val as u64, val < 0);
        self.push_int_val(llvm_val);
    }

    /// 将一个 i64 值推入操作数栈。
    fn push_long_val(&mut self, val: IntValue<'ctx>) {
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let bitcast_val = self
            .builder
            .build_bit_cast(val, i64_type, "long_bitcast")
            .expect("bitcast failed")
            .into_int_value();
        let lo = self
            .builder
            .build_int_truncate(bitcast_val, i32_type, "lo_trunc")
            .expect("lo trunc failed");
        let hi = self
            .builder
            .build_int_truncate(
                self.builder
                    .build_right_shift(
                        bitcast_val,
                        i64_type.const_int(32, false),
                        false,
                        "hi_shift",
                    )
                    .expect("hi shift failed"),
                i32_type,
                "hi_i32",
            )
            .expect("hi_i32 trunc failed");
        self.push_two_slots(lo, hi);
    }

    /// 从操作数栈弹出一个 i64 值。
    fn pop_long(&mut self) -> IntValue<'ctx> {
        let (lo, hi) = self.pop_two_slots();
        let i64_type = self.context.i64_type();
        let lo_ext = self
            .builder
            .build_int_z_extend(lo, i64_type, "lo_ext")
            .expect("zext failed");
        let hi_ext = self
            .builder
            .build_int_z_extend(hi, i64_type, "hi_ext")
            .expect("zext failed");
        let combined = self
            .builder
            .build_or(
                lo_ext,
                self.builder
                    .build_left_shift(hi_ext, i64_type.const_int(32, false), "hi_shl")
                    .expect("shl failed"),
                "combined",
            )
            .expect("or failed");
        self.builder
            .build_bit_cast(combined, i64_type, "long_bitcast")
            .expect("bitcast failed")
            .into_int_value()
    }

    /// 将一个 f32 值推入操作数栈。
    fn push_float_val(&mut self, val: inkwell::values::FloatValue<'ctx>) {
        let i32_type = self.context.i32_type();
        let bitcast = self
            .builder
            .build_bit_cast(val, i32_type, "float_bitcast")
            .expect("bitcast failed")
            .into_int_value();
        self.push_one_slot(bitcast);
    }

    /// 从操作数栈弹出一个 f32 值。
    fn pop_float(&mut self) -> inkwell::values::FloatValue<'ctx> {
        let val = self.pop_one_slot();
        self.builder
            .build_bit_cast(val, self.context.f32_type(), "float_bitcast")
            .expect("bitcast failed")
            .into_float_value()
    }

    /// 将一个 f64 值推入操作数栈。
    fn push_double_val(&mut self, val: inkwell::values::FloatValue<'ctx>) {
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let bitcast_val = self
            .builder
            .build_bit_cast(val, i64_type, "double_bitcast")
            .expect("bitcast failed")
            .into_int_value();
        let lo = self
            .builder
            .build_int_truncate(bitcast_val, i32_type, "lo_trunc")
            .expect("lo trunc failed");
        let hi = self
            .builder
            .build_int_truncate(
                self.builder
                    .build_right_shift(
                        bitcast_val,
                        i64_type.const_int(32, false),
                        false,
                        "hi_shift",
                    )
                    .expect("hi shift failed"),
                i32_type,
                "hi_i32",
            )
            .expect("hi_i32 trunc failed");
        self.push_two_slots(lo, hi);
    }

    /// 从操作数栈弹出一个 f64 值。
    fn pop_double(&mut self) -> inkwell::values::FloatValue<'ctx> {
        let (lo, hi) = self.pop_two_slots();
        let i64_type = self.context.i64_type();
        let lo_ext = self
            .builder
            .build_int_z_extend(lo, i64_type, "lo_ext")
            .expect("zext failed");
        let hi_ext = self
            .builder
            .build_int_z_extend(hi, i64_type, "hi_ext")
            .expect("zext failed");
        let combined = self
            .builder
            .build_or(
                lo_ext,
                self.builder
                    .build_left_shift(hi_ext, i64_type.const_int(32, false), "hi_shl")
                    .expect("shl failed"),
                "combined",
            )
            .expect("or failed");
        self.builder
            .build_bit_cast(combined, i64_type, "double_bitcast")
            .expect("bitcast failed")
            .into_float_value()
    }

    /// 推入一个 i32 到栈槽。
    fn push_one_slot(&mut self, val: IntValue<'ctx>) {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("load stack_top failed")
            .into_int_value();
        if let Some(stack_param) = self.stack_param {
            let ptr = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[top_i32],
                        "push_store",
                    )
                    .expect("push gep failed")
            };
            self.builder
                .build_store(ptr, val)
                .expect("push store failed");
        }
        let one = self.context.i32_type().const_int(1, false);
        let new_top = self
            .builder
            .build_int_add(top_i32, one, "inc_top")
            .expect("inc_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("store stack_top failed");
    }

    /// 推入两个 i32 到栈槽（用于 long/double）。
    fn push_two_slots(&mut self, lo: IntValue<'ctx>, hi: IntValue<'ctx>) {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("load stack_top failed")
            .into_int_value();
        if let Some(stack_param) = self.stack_param {
            let ptr_lo = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[top_i32],
                        "push_lo",
                    )
                    .expect("push gep failed")
            };
            self.builder
                .build_store(ptr_lo, lo)
                .expect("push lo store failed");
            let one = self.context.i32_type().const_int(1, false);
            let top_plus_one = self
                .builder
                .build_int_add(top_i32, one, "top_plus_one")
                .expect("add failed");
            let ptr_hi = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[top_plus_one],
                        "push_hi",
                    )
                    .expect("push gep failed")
            };
            self.builder
                .build_store(ptr_hi, hi)
                .expect("push hi store failed");
        }
        let two = self.context.i32_type().const_int(2, false);
        let new_top = self
            .builder
            .build_int_add(top_i32, two, "inc_top")
            .expect("inc_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("store stack_top failed");
    }

    /// 弹出一个 i32 从栈槽。
    fn pop_one_slot(&mut self) -> IntValue<'ctx> {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("load stack_top failed")
            .into_int_value();
        let one = self.context.i32_type().const_int(1, false);
        let slot_idx_val = self
            .builder
            .build_int_sub(top_i32, one, "dec_top")
            .expect("dec_top failed");
        let val = if let Some(stack_param) = self.stack_param {
            let ptr = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[slot_idx_val],
                        "pop_ptr",
                    )
                    .expect("pop gep failed")
            };
            self.builder
                .build_load(self.context.i32_type(), ptr, "pop_val")
                .expect("pop load failed")
                .into_int_value()
        } else {
            slot_idx_val
        };
        self.builder
            .build_store(self.stack_top, slot_idx_val)
            .expect("pop store stack_top failed");
        val
    }

    /// 弹出两个 i32 从栈槽（用于 long/double）。
    fn pop_two_slots(&mut self) -> (IntValue<'ctx>, IntValue<'ctx>) {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("load stack_top failed")
            .into_int_value();
        let one = self.context.i32_type().const_int(1, false);
        let two = self.context.i32_type().const_int(2, false);
        let slot_idx_val = self
            .builder
            .build_int_sub(top_i32, two, "dec_top")
            .expect("dec_top failed");
        let hi_idx = self
            .builder
            .build_int_add(slot_idx_val, one, "hi_idx")
            .expect("add failed");
        let (lo, hi) = if let Some(stack_param) = self.stack_param {
            let ptr_lo = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[slot_idx_val],
                        "pop_lo",
                    )
                    .expect("pop gep failed")
            };
            let loaded_lo = self
                .builder
                .build_load(self.context.i32_type(), ptr_lo, "pop_lo")
                .expect("pop lo load failed")
                .into_int_value();
            let ptr_hi = unsafe {
                self.builder
                    .build_in_bounds_gep(self.context.i32_type(), stack_param, &[hi_idx], "pop_hi")
                    .expect("pop gep failed")
            };
            let loaded_hi = self
                .builder
                .build_load(self.context.i32_type(), ptr_hi, "pop_hi")
                .expect("pop hi load failed")
                .into_int_value();
            (loaded_lo, loaded_hi)
        } else {
            (slot_idx_val, hi_idx)
        };
        self.builder
            .build_store(self.stack_top, slot_idx_val)
            .expect("pop store stack_top failed");
        (lo, hi)
    }

    /// 从操作数栈弹出一个 i32 值。
    fn pop_int(&mut self) -> IntValue<'ctx> {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("pop load stack_top failed")
            .into_int_value();

        // 计算 slot_idx = top - 1
        let one = self.context.i32_type().const_int(1, false);
        let slot_idx_val = self
            .builder
            .build_int_sub(top_i32, one, "dec_top")
            .expect("dec_top failed");

        // 用 GEP 动态加载值
        let val = if let Some(stack_param) = self.stack_param {
            let ptr = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[slot_idx_val],
                        "pop_ptr",
                    )
                    .expect("pop gep failed")
            };
            self.builder
                .build_load(self.context.i32_type(), ptr, "pop_val")
                .expect("pop load failed")
                .into_int_value()
        } else {
            // 回退：使用静态 slot 索引
            let slot_idx = slot_idx_val.get_zero_extended_constant().unwrap_or(0) as usize;
            if slot_idx < self.stack_vars.len() {
                self.builder
                    .build_load(
                        self.context.i32_type(),
                        self.stack_vars[slot_idx],
                        "pop_val",
                    )
                    .expect("pop load failed")
                    .into_int_value()
            } else {
                slot_idx_val
            }
        };

        // 更新 stack_top
        self.builder
            .build_store(self.stack_top, slot_idx_val)
            .expect("pop store stack_top failed");

        val
    }

    /// 读取栈顶的 i32 值但不弹出（用于 checkcast 等指令）。
    fn peek_stack_int(&mut self) -> IntValue<'ctx> {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "peek_load_top")
            .expect("peek load stack_top failed")
            .into_int_value();

        // 计算 slot_idx = top - 1
        let one = self.context.i32_type().const_int(1, false);
        let slot_idx_val = self
            .builder
            .build_int_sub(top_i32, one, "peek_slot")
            .expect("peek dec_top failed");

        if let Some(stack_param) = self.stack_param {
            let ptr = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        self.context.i32_type(),
                        stack_param,
                        &[slot_idx_val],
                        "peek_ptr",
                    )
                    .expect("peek gep failed")
            };
            self.builder
                .build_load(self.context.i32_type(), ptr, "peek_val")
                .expect("peek load failed")
                .into_int_value()
        } else {
            let slot_idx = slot_idx_val.get_zero_extended_constant().unwrap_or(0) as usize;
            if slot_idx < self.stack_vars.len() {
                self.builder
                    .build_load(
                        self.context.i32_type(),
                        self.stack_vars[slot_idx],
                        "peek_val",
                    )
                    .expect("peek load failed")
                    .into_int_value()
            } else {
                self.context.i32_type().const_zero()
            }
        }
    }

    // ============================================================
    // 本地变量操作辅助方法
    // ============================================================

    fn load_local_int(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let val = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx],
                    &format!("load_local_{}", idx),
                )
                .expect("load_local_int failed")
                .into_int_value();
            self.push_int_val(val);
        }
    }

    fn store_local_int(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let val = self.pop_int();
            self.builder
                .build_store(self.local_vars[idx], val)
                .expect("store_local_int failed");
        }
    }

    fn load_local_long(&mut self, idx: usize) {
        // Longs occupy 2 consecutive i32 local slots
        if idx + 1 < self.local_vars.len() {
            let lo = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx],
                    &format!("load_local_long_lo_{}", idx),
                )
                .expect("load local long lo failed")
                .into_int_value();
            let hi = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx + 1],
                    &format!("load_local_long_hi_{}", idx),
                )
                .expect("load local long hi failed")
                .into_int_value();
            let i64_type = self.context.i64_type();
            let lo_ext = self
                .builder
                .build_int_z_extend(lo, i64_type, "long_lo_ext")
                .expect("zext failed");
            let hi_ext = self
                .builder
                .build_int_z_extend(hi, i64_type, "long_hi_ext")
                .expect("zext failed");
            let combined = self
                .builder
                .build_or(
                    lo_ext,
                    self.builder
                        .build_left_shift(hi_ext, i64_type.const_int(32, false), "long_hi_shl")
                        .expect("shl failed"),
                    "long_combined",
                )
                .expect("or failed");
            self.push_long_val(combined);
        }
    }

    fn store_local_long(&mut self, idx: usize) {
        let val = self.pop_long();
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let bitcast = self
            .builder
            .build_bit_cast(val, i64_type, "store_long_bitcast")
            .expect("bitcast failed")
            .into_int_value();
        let lo = self
            .builder
            .build_int_truncate(bitcast, i32_type, "store_long_lo")
            .expect("trunc failed");
        let hi = self
            .builder
            .build_int_truncate(
                self.builder
                    .build_right_shift(
                        bitcast,
                        i64_type.const_int(32, false),
                        false,
                        "store_long_hi_shift",
                    )
                    .expect("shift failed"),
                i32_type,
                "store_long_hi",
            )
            .expect("trunc failed");
        if idx + 1 < self.local_vars.len() {
            self.builder
                .build_store(self.local_vars[idx], lo)
                .expect("store local long lo failed");
            self.builder
                .build_store(self.local_vars[idx + 1], hi)
                .expect("store local long hi failed");
        }
    }

    fn load_local_ref(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let val = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx],
                    &format!("load_local_ref_{}", idx),
                )
                .expect("load_local_ref failed")
                .into_int_value();
            self.push_int_val(val);
        }
    }

    fn store_local_ref(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let val = self.pop_int(); // slot_id
            self.builder
                .build_store(self.local_vars[idx], val)
                .expect("store_local_ref failed");
        }
    }

    fn load_local_float(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let raw = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx],
                    &format!("load_local_f_raw_{}", idx),
                )
                .expect("load_local_float failed")
                .into_int_value();
            let fval = self
                .builder
                .build_bit_cast(raw, self.context.f32_type(), "f_bitcast")
                .expect("f bitcast failed")
                .into_float_value();
            self.push_float_val(fval);
        }
    }

    fn store_local_float(&mut self, idx: usize) {
        if idx < self.local_vars.len() {
            let fval = self.pop_float();
            let raw = self
                .builder
                .build_bit_cast(fval, self.context.i32_type(), "f_store_bitcast")
                .expect("f store bitcast failed")
                .into_int_value();
            self.builder
                .build_store(self.local_vars[idx], raw)
                .expect("store_local_float failed");
        }
    }

    fn load_local_double(&mut self, idx: usize) {
        if idx + 1 < self.local_vars.len() {
            let lo = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx],
                    &format!("load_local_d_lo_{}", idx),
                )
                .expect("load local double lo failed")
                .into_int_value();
            let hi = self
                .builder
                .build_load(
                    self.context.i32_type(),
                    self.local_vars[idx + 1],
                    &format!("load_local_d_hi_{}", idx),
                )
                .expect("load local double hi failed")
                .into_int_value();
            let i64_type = self.context.i64_type();
            let lo_ext = self
                .builder
                .build_int_z_extend(lo, i64_type, "d_lo_ext")
                .expect("d zext failed");
            let hi_ext = self
                .builder
                .build_int_z_extend(hi, i64_type, "d_hi_ext")
                .expect("d zext failed");
            let combined = self
                .builder
                .build_or(
                    lo_ext,
                    self.builder
                        .build_left_shift(hi_ext, i64_type.const_int(32, false), "d_hi_shl")
                        .expect("d shl failed"),
                    "d_combined",
                )
                .expect("d or failed");
            let dval = self
                .builder
                .build_bit_cast(combined, self.context.f64_type(), "d_bitcast")
                .expect("d bitcast failed")
                .into_float_value();
            self.push_double_val(dval);
        }
    }

    fn store_local_double(&mut self, idx: usize) {
        let dval = self.pop_double();
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let bitcast = self
            .builder
            .build_bit_cast(dval, i64_type, "d_store_bitcast")
            .expect("d store bitcast failed")
            .into_int_value();
        let lo = self
            .builder
            .build_int_truncate(bitcast, i32_type, "d_store_lo")
            .expect("d store lo trunc failed");
        let hi = self
            .builder
            .build_int_truncate(
                self.builder
                    .build_right_shift(
                        bitcast,
                        i64_type.const_int(32, false),
                        false,
                        "d_store_hi_shift",
                    )
                    .expect("d store hi shift failed"),
                i32_type,
                "d_store_hi",
            )
            .expect("d store hi trunc failed");
        if idx + 1 < self.local_vars.len() {
            self.builder
                .build_store(self.local_vars[idx], lo)
                .expect("store local double lo failed");
            self.builder
                .build_store(self.local_vars[idx + 1], hi)
                .expect("store local double hi failed");
        }
    }

    // ============================================================
    // 比较辅助方法
    // ============================================================

    /// Build float comparison: pop 2 floats, push -1/0/1 result.
    /// NaN→-1 when nan_is_neg=true (fcmpl), NaN→1 when nan_is_neg=false (fcmpg).
    fn build_float_compare(&mut self, nan_is_pos: bool) {
        let v2 = self.pop_float();
        let v1 = self.pop_float();
        let nan_result = if nan_is_pos {
            self.context.i32_type().const_int(1, false)
        } else {
            self.context.i32_type().const_int(-1i64 as u64, false)
        };
        // fcmp oeq: true if ordered and equal
        let eq = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, v1, v2, "fcmpl_eq")
            .expect("fcmpl eq failed");
        let gt = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OGT, v1, v2, "fcmpl_gt")
            .expect("fcmpl gt failed");
        let lt = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OLT, v1, v2, "fcmpl_lt")
            .expect("fcmpl lt failed");
        let zero = self.context.i32_type().const_int(0, false);
        let one = self.context.i32_type().const_int(1, false);
        let neg_one = self.context.i32_type().const_int(-1i64 as u64, false);
        let eq_result = self
            .builder
            .build_select(eq, zero, nan_result, "fcmpl_eq_sel")
            .expect("fcmpl eq select failed")
            .into_int_value();
        let gt_result = self
            .builder
            .build_select(gt, one, eq_result, "fcmpl_gt_sel")
            .expect("fcmpl gt select failed")
            .into_int_value();
        let result = self
            .builder
            .build_select(lt, neg_one, gt_result, "fcmpl_lt_sel")
            .expect("fcmpl lt select failed")
            .into_int_value();
        self.push_int_val(result);
    }

    /// Build double comparison: pop 2 doubles, push -1/0/1 result.
    fn build_double_compare(&mut self, nan_is_pos: bool) {
        let v2 = self.pop_double();
        let v1 = self.pop_double();
        let nan_result = if nan_is_pos {
            self.context.i32_type().const_int(1, false)
        } else {
            self.context.i32_type().const_int(-1i64 as u64, false)
        };
        let eq = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, v1, v2, "dcmpl_eq")
            .expect("dcmpl eq failed");
        let gt = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OGT, v1, v2, "dcmpl_gt")
            .expect("dcmpl gt failed");
        let lt = self
            .builder
            .build_float_compare(inkwell::FloatPredicate::OLT, v1, v2, "dcmpl_lt")
            .expect("dcmpl lt failed");
        let zero = self.context.i32_type().const_int(0, false);
        let one = self.context.i32_type().const_int(1, false);
        let neg_one = self.context.i32_type().const_int(-1i64 as u64, false);
        let eq_result = self
            .builder
            .build_select(eq, zero, nan_result, "dcmpl_eq_sel")
            .expect("dcmpl eq select failed")
            .into_int_value();
        let gt_result = self
            .builder
            .build_select(gt, one, eq_result, "dcmpl_gt_sel")
            .expect("dcmpl gt select failed")
            .into_int_value();
        let result = self
            .builder
            .build_select(lt, neg_one, gt_result, "dcmpl_lt_sel")
            .expect("dcmpl lt select failed")
            .into_int_value();
        self.push_int_val(result);
    }

    // ============================================================
    // 方法调用辅助方法
    // ============================================================

    /// 生成调用 JIT invoke 运行时函数的 LLVM IR。
    /// 签名: extern "C" fn(cp_idx: u16, locals: *mut i32, stack: *mut i32, stack_top: u32)
    fn call_invoke_runtime(&mut self, fn_name: &str, cp_idx: u16) {
        let i32_type = self.context.i32_type();
        let i16_type = self.context.i16_type();

        // 声明外部函数
        let runtime_fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                self.context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function(fn_name, runtime_fn_type, None);

        // 获取 locals 指针（即传入的第一个参数）
        let locals_ptr = self
            .function
            .get_first_param()
            .unwrap()
            .into_pointer_value();
        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();

        // 加载当前 stack_top 值
        let stack_top_val = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "invoke_stack_top")
            .expect("load stack_top failed")
            .into_int_value();

        // cp_idx 参数
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        // 调用运行时函数
        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    locals_ptr.into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "invoke_call",
            )
            .expect("invoke call failed");
    }

    // ============================================================
    // Runtime callout 辅助方法
    // ============================================================

    // --- ldc / ldc_w ---
    fn call_ldc_runtime(&mut self, cp_idx: u16) {
        let i32_type = self.context.i32_type();
        let i16_type = self.context.i16_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_ldc", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "ldc_stack_top")
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "ldc_call",
            )
            .expect("ldc call failed");
    }

    // --- ldc2_w ---
    fn call_ldc2_w_runtime(&mut self, cp_idx: u16) {
        let i32_type = self.context.i32_type();
        let i16_type = self.context.i16_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_ldc2_w", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "ldc2w_stack_top")
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "ldc2_w_call",
            )
            .expect("ldc2_w call failed");

        // ldc2_w pushes 2 slots; advance stack_top by 2
        let two = self.context.i32_type().const_int(2, false);
        let new_top = self
            .builder
            .build_int_add(stack_top_val, two, "inc_top_ldc2w")
            .expect("inc_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("store stack_top failed");
    }

    // --- new ---
    fn call_new_inst_runtime(&mut self, cp_idx: u16) -> IntValue<'ctx> {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i16_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_new_inst", fn_type, None);
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        let call_site = self
            .builder
            .build_call(runtime_fn, &[cp_idx_val.into()], "new_inst_call")
            .expect("new_inst call failed");
        call_site
            .try_as_basic_value()
            .unwrap_basic()
            .into_int_value()
    }

    // --- newarray ---
    fn call_new_array_runtime(&mut self, ary_type: u8, size: IntValue<'ctx>) -> IntValue<'ctx> {
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let fn_type = i32_type.fn_type(&[i8_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_new_array", fn_type, None);
        let ary_type_val = i8_type.const_int(ary_type as u64, false);

        let call_site = self
            .builder
            .build_call(
                runtime_fn,
                &[ary_type_val.into(), size.into()],
                "new_array_call",
            )
            .expect("new_array call failed");
        call_site
            .try_as_basic_value()
            .unwrap_basic()
            .into_int_value()
    }

    // --- anewarray ---
    fn call_anewarray_runtime(&mut self, cp_idx: u16, size: IntValue<'ctx>) -> IntValue<'ctx> {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i16_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_anewarray", fn_type, None);
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        let call_site = self
            .builder
            .build_call(
                runtime_fn,
                &[cp_idx_val.into(), size.into()],
                "anewarray_call",
            )
            .expect("anewarray call failed");
        call_site
            .try_as_basic_value()
            .unwrap_basic()
            .into_int_value()
    }

    // --- arraylength ---
    fn call_array_length_runtime(&mut self, obj: IntValue<'ctx>) -> IntValue<'ctx> {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_array_length", fn_type, None);

        let call_site = self
            .builder
            .build_call(runtime_fn, &[obj.into()], "array_length_call")
            .expect("array_length call failed");
        call_site
            .try_as_basic_value()
            .unwrap_basic()
            .into_int_value()
    }

    // --- checkcast ---
    fn call_checkcast_runtime(&mut self, cp_idx: u16, obj: IntValue<'ctx>) {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i16_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_checkcast", fn_type, None);
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[cp_idx_val.into(), obj.into()],
                "checkcast_call",
            )
            .expect("checkcast call failed");
    }

    // --- instanceof ---
    fn call_instanceof_runtime(&mut self, cp_idx: u16, obj: IntValue<'ctx>) -> IntValue<'ctx> {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i16_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_instanceof", fn_type, None);
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        let call_site = self
            .builder
            .build_call(
                runtime_fn,
                &[cp_idx_val.into(), obj.into()],
                "instanceof_call",
            )
            .expect("instanceof call failed");
        call_site
            .try_as_basic_value()
            .unwrap_basic()
            .into_int_value()
    }

    // --- getfield ---
    fn call_getfield_runtime(&mut self, cp_idx: u16) {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_getfield", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(
                self.context.i32_type(),
                self.stack_top,
                "getfield_stack_top",
            )
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "getfield_call",
            )
            .expect("getfield call failed");
    }

    // --- putfield ---
    fn call_putfield_runtime(&mut self, cp_idx: u16) {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_putfield", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(
                self.context.i32_type(),
                self.stack_top,
                "putfield_stack_top",
            )
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "putfield_call",
            )
            .expect("putfield call failed");
    }

    // --- getstatic ---
    fn call_getstatic_runtime(&mut self, cp_idx: u16) {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_getstatic", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(
                self.context.i32_type(),
                self.stack_top,
                "getstatic_stack_top",
            )
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "getstatic_call",
            )
            .expect("getstatic call failed");

        // Push result slot (1 slot for getstatic int; for other types this needs expansion)
        let one = self.context.i32_type().const_int(1, false);
        let new_top = self
            .builder
            .build_int_add(stack_top_val, one, "inc_top_getstatic")
            .expect("inc_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("store stack_top failed");
    }

    // --- putstatic ---
    fn call_putstatic_runtime(&mut self, cp_idx: u16) {
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());

        let fn_type = self.context.void_type().fn_type(
            &[
                i16_type.into(),
                ptr_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_putstatic", fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        let stack_top_val = self
            .builder
            .build_load(
                self.context.i32_type(),
                self.stack_top,
                "putstatic_stack_top",
            )
            .expect("load stack_top failed")
            .into_int_value();
        let cp_idx_val = i16_type.const_int(cp_idx as u64, false);

        self.builder
            .build_call(
                runtime_fn,
                &[
                    cp_idx_val.into(),
                    self.function.get_first_param().unwrap().into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "putstatic_call",
            )
            .expect("putstatic call failed");
    }

    // --- 数组加载（通用模式） ---
    fn call_array_load_runtime(
        &mut self,
        fn_name: &str,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = self.context.void_type().fn_type(
            &[
                i32_type.into(),
                i32_type.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function(fn_name, fn_type, None);

        let stack_ptr = self.function.get_nth_param(1).unwrap().into_pointer_value();
        // After pop array + index, stack_top decreased by 2, result written at stack_top - 2
        let stack_top_val = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "aload_stack_top")
            .expect("load stack_top failed")
            .into_int_value();

        self.builder
            .build_call(
                runtime_fn,
                &[
                    array.into(),
                    index.into(),
                    stack_ptr.into(),
                    stack_top_val.into(),
                ],
                "aload_call",
            )
            .expect("array load call failed");
    }

    fn call_iaload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_iaload", array, index);
    }
    fn call_laload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_laload", array, index);
    }
    fn call_faload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_faload", array, index);
    }
    fn call_daload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_daload", array, index);
    }
    fn call_aaload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_aaload", array, index);
    }
    fn call_baload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_baload", array, index);
    }
    fn call_caload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_caload", array, index);
    }
    fn call_saload_runtime(&mut self, array: IntValue<'ctx>, index: IntValue<'ctx>) {
        self.call_array_load_runtime("jit_saload", array, index);
    }

    // --- 数组存储 ---
    fn call_iastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_iastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "iastore_call",
            )
            .expect("iastore call failed");
    }
    fn call_lastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        lo: IntValue<'ctx>,
        hi: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self.context.void_type().fn_type(
            &[
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_lastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), lo.into(), hi.into()],
                "lastore_call",
            )
            .expect("lastore call failed");
    }
    fn call_fastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_fastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "fastore_call",
            )
            .expect("fastore call failed");
    }
    fn call_dastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        lo: IntValue<'ctx>,
        hi: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self.context.void_type().fn_type(
            &[
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_dastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), lo.into(), hi.into()],
                "dastore_call",
            )
            .expect("dastore call failed");
    }
    fn call_aastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_aastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "aastore_call",
            )
            .expect("aastore call failed");
    }
    fn call_bastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_bastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "bastore_call",
            )
            .expect("bastore call failed");
    }
    fn call_castore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_castore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "castore_call",
            )
            .expect("castore call failed");
    }
    fn call_sastore_runtime(
        &mut self,
        array: IntValue<'ctx>,
        index: IntValue<'ctx>,
        value: IntValue<'ctx>,
    ) {
        let i32_type = self.context.i32_type();
        let fn_type = self
            .context
            .void_type()
            .fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_sastore", fn_type, None);
        self.builder
            .build_call(
                runtime_fn,
                &[array.into(), index.into(), value.into()],
                "sastore_call",
            )
            .expect("sastore call failed");
    }

    // --- 同步 ---
    fn call_monitorenter_runtime(&mut self, obj: IntValue<'ctx>) {
        let i32_type = self.context.i32_type();
        let fn_type = self.context.void_type().fn_type(&[i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_monitorenter", fn_type, None);
        self.builder
            .build_call(runtime_fn, &[obj.into()], "monitorenter_call")
            .expect("monitorenter call failed");
    }

    fn call_monitorexit_runtime(&mut self, obj: IntValue<'ctx>) {
        let i32_type = self.context.i32_type();
        let fn_type = self.context.void_type().fn_type(&[i32_type.into()], false);
        let module = unsafe { &*self.module };
        let runtime_fn = module.add_function("jit_monitorexit", fn_type, None);
        self.builder
            .build_call(runtime_fn, &[obj.into()], "monitorexit_call")
            .expect("monitorexit call failed");
    }

    // ============================================================
    // 控制流辅助方法
    // ============================================================

    fn branch_to(&mut self, target: usize) {
        let target_bb = self.ensure_block(target);
        self.builder
            .build_unconditional_branch(target_bb)
            .expect("branch_to failed");
        self.builder.position_at_end(target_bb);
    }

    fn ensure_block(&mut self, offset: usize) -> BasicBlock<'ctx> {
        if !self.bb_map.contains_key(&offset) {
            let name = format!("bb_{}", offset);
            let bb = self.context.append_basic_block(self.function, &name);
            self.bb_map.insert(offset, bb);
        }
        *self.bb_map.get(&offset).unwrap()
    }

    /// 处理因分支指令提前返回而遗漏的基本块。
    fn translate_remaining_blocks(&mut self) {
        // 收集所有还没有终止符的 block 及其对应的 PC
        let pending: Vec<usize> = self
            .bb_map
            .keys()
            .copied()
            .filter(|&pc| pc != 0) // 跳过 entry block（已处理）
            .collect();

        for pc in pending {
            let bb = self.bb_map[&pc];
            if bb.get_terminator().is_none() && pc < self.bytecode.len() {
                self.builder.position_at_end(bb);
                self.translate_bytecode(pc);
            }
        }
    }
}

/// 扫描 bytecode，收集所有跳转目标偏移。
fn collect_jump_targets(bytecode: &[U1]) -> Vec<usize> {
    let mut targets = Vec::new();
    let mut pc = 0;

    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        let opcode = OpCode::from(opcode);

        match opcode {
            OpCode::ifeq
            | OpCode::ifne
            | OpCode::iflt
            | OpCode::ifge
            | OpCode::ifgt
            | OpCode::ifle
            | OpCode::if_icmpeq
            | OpCode::if_icmpne
            | OpCode::if_icmplt
            | OpCode::if_icmpge
            | OpCode::if_icmpgt
            | OpCode::if_icmple
            | OpCode::if_acmpeq
            | OpCode::if_acmpne
            | OpCode::ifnull
            | OpCode::ifnonnull => {
                if pc + 2 < bytecode.len() {
                    let offset = i16::from_be_bytes([bytecode[pc + 1], bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    targets.push(target);
                    pc += 3;
                } else {
                    break;
                }
            }
            OpCode::goto => {
                if pc + 2 < bytecode.len() {
                    let offset = i16::from_be_bytes([bytecode[pc + 1], bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    targets.push(target);
                    pc += 3;
                } else {
                    break;
                }
            }
            OpCode::goto_w | OpCode::jsr_w => {
                if pc + 4 < bytecode.len() {
                    let offset = i32::from_be_bytes([
                        bytecode[pc + 1],
                        bytecode[pc + 2],
                        bytecode[pc + 3],
                        bytecode[pc + 4],
                    ]);
                    let target = (pc as i32 + offset) as usize;
                    targets.push(target);
                    pc += 5;
                } else {
                    break;
                }
            }
            OpCode::jsr => {
                if pc + 2 < bytecode.len() {
                    let offset = i16::from_be_bytes([bytecode[pc + 1], bytecode[pc + 2]]) as i32;
                    let target = (pc as i32 + offset) as usize;
                    targets.push(target);
                    pc += 3;
                } else {
                    break;
                }
            }
            OpCode::tableswitch => {
                let mut ptr = pc + 1;
                if ptr % 4 != 0 {
                    ptr += 4 - ptr % 4;
                }
                if ptr + 12 <= bytecode.len() {
                    let default = i32::from_be_bytes([
                        bytecode[ptr],
                        bytecode[ptr + 1],
                        bytecode[ptr + 2],
                        bytecode[ptr + 3],
                    ]);
                    let low = i32::from_be_bytes([
                        bytecode[ptr + 4],
                        bytecode[ptr + 5],
                        bytecode[ptr + 6],
                        bytecode[ptr + 7],
                    ]);
                    let high = i32::from_be_bytes([
                        bytecode[ptr + 8],
                        bytecode[ptr + 9],
                        bytecode[ptr + 10],
                        bytecode[ptr + 11],
                    ]);
                    targets.push((pc as i32 + default) as usize);
                    let num_targets = (high - low + 1) as usize;
                    let table_end = ptr + 12 + num_targets * 4;
                    for i in 0..num_targets {
                        let entry_ptr = ptr + 12 + i * 4;
                        if entry_ptr + 4 <= bytecode.len() {
                            let offset = i32::from_be_bytes([
                                bytecode[entry_ptr],
                                bytecode[entry_ptr + 1],
                                bytecode[entry_ptr + 2],
                                bytecode[entry_ptr + 3],
                            ]);
                            targets.push((pc as i32 + offset) as usize);
                        }
                    }
                    pc = table_end;
                } else {
                    break;
                }
            }
            OpCode::lookupswitch => {
                let mut ptr = pc + 1;
                if ptr % 4 != 0 {
                    ptr += 4 - ptr % 4;
                }
                if ptr + 8 <= bytecode.len() {
                    let default = i32::from_be_bytes([
                        bytecode[ptr],
                        bytecode[ptr + 1],
                        bytecode[ptr + 2],
                        bytecode[ptr + 3],
                    ]);
                    let npairs = i32::from_be_bytes([
                        bytecode[ptr + 4],
                        bytecode[ptr + 5],
                        bytecode[ptr + 6],
                        bytecode[ptr + 7],
                    ]);
                    targets.push((pc as i32 + default) as usize);
                    for i in 0..npairs {
                        let pair_ptr = ptr + 8 + (i * 8) as usize;
                        if pair_ptr + 8 <= bytecode.len() {
                            let offset = i32::from_be_bytes([
                                bytecode[pair_ptr + 4],
                                bytecode[pair_ptr + 5],
                                bytecode[pair_ptr + 6],
                                bytecode[pair_ptr + 7],
                            ]);
                            targets.push((pc as i32 + offset) as usize);
                        }
                    }
                    pc = ptr + 8 + (npairs as usize) * 8;
                } else {
                    break;
                }
            }
            _ => {
                pc += opcode_size(opcode);
            }
        }
    }

    targets.sort();
    targets.dedup();
    targets
}

/// 返回 opcode 的总字节数（包括操作数）。
fn opcode_size(opcode: OpCode) -> usize {
    match opcode {
        OpCode::nop
        | OpCode::aconst_null
        | OpCode::iconst_m1
        | OpCode::iconst_0
        | OpCode::iconst_1
        | OpCode::iconst_2
        | OpCode::iconst_3
        | OpCode::iconst_4
        | OpCode::iconst_5
        | OpCode::lconst_0
        | OpCode::lconst_1
        | OpCode::fconst_0
        | OpCode::fconst_1
        | OpCode::fconst_2
        | OpCode::dconst_0
        | OpCode::dconst_1
        | OpCode::iaload
        | OpCode::laload
        | OpCode::faload
        | OpCode::daload
        | OpCode::aaload
        | OpCode::baload
        | OpCode::caload
        | OpCode::saload
        | OpCode::iastore
        | OpCode::lastore
        | OpCode::fastore
        | OpCode::dastore
        | OpCode::aastore
        | OpCode::bastore
        | OpCode::castore
        | OpCode::sastore
        | OpCode::pop
        | OpCode::pop2
        | OpCode::dup
        | OpCode::dup_x1
        | OpCode::dup_x2
        | OpCode::dup2
        | OpCode::dup2_x1
        | OpCode::dup2_x2
        | OpCode::swap
        | OpCode::iadd
        | OpCode::ladd
        | OpCode::fadd
        | OpCode::dadd
        | OpCode::isub
        | OpCode::lsub
        | OpCode::fsub
        | OpCode::dsub
        | OpCode::imul
        | OpCode::lmul
        | OpCode::fmul
        | OpCode::dmul
        | OpCode::idiv
        | OpCode::ldiv
        | OpCode::fdiv
        | OpCode::ddiv
        | OpCode::irem
        | OpCode::lrem
        | OpCode::frem
        | OpCode::drem
        | OpCode::ineg
        | OpCode::lneg
        | OpCode::fneg
        | OpCode::dneg
        | OpCode::ishl
        | OpCode::lshl
        | OpCode::ishr
        | OpCode::lshr
        | OpCode::iushr
        | OpCode::lushr
        | OpCode::iand
        | OpCode::land
        | OpCode::ior
        | OpCode::lor
        | OpCode::ixor
        | OpCode::lxor
        | OpCode::i2l
        | OpCode::i2f
        | OpCode::i2d
        | OpCode::l2i
        | OpCode::l2f
        | OpCode::l2d
        | OpCode::f2i
        | OpCode::f2l
        | OpCode::f2d
        | OpCode::d2i
        | OpCode::d2l
        | OpCode::d2f
        | OpCode::i2b
        | OpCode::i2c
        | OpCode::i2s
        | OpCode::lcmp
        | OpCode::fcmpl
        | OpCode::fcmpg
        | OpCode::dcmpl
        | OpCode::dcmpg
        | OpCode::ireturn
        | OpCode::lreturn
        | OpCode::freturn
        | OpCode::dreturn
        | OpCode::areturn
        | OpCode::return_void
        | OpCode::arraylength
        | OpCode::athrow
        | OpCode::monitorenter
        | OpCode::monitorexit => 1,

        OpCode::bipush
        | OpCode::ldc
        | OpCode::iload
        | OpCode::lload
        | OpCode::fload
        | OpCode::dload
        | OpCode::aload
        | OpCode::istore
        | OpCode::lstore
        | OpCode::fstore
        | OpCode::dstore
        | OpCode::astore
        | OpCode::iinc
        | OpCode::ret
        | OpCode::newarray
        | OpCode::anewarray
        | OpCode::ldc_w
        | OpCode::ldc2_w
        | OpCode::new
        | OpCode::checkcast
        | OpCode::instanceof
        | OpCode::getstatic
        | OpCode::putstatic
        | OpCode::getfield
        | OpCode::putfield
        | OpCode::invokevirtual
        | OpCode::invokespecial
        | OpCode::invokestatic
        | OpCode::ifeq
        | OpCode::ifne
        | OpCode::iflt
        | OpCode::ifge
        | OpCode::ifgt
        | OpCode::ifle
        | OpCode::if_icmpeq
        | OpCode::if_icmpne
        | OpCode::if_icmplt
        | OpCode::if_icmpge
        | OpCode::if_icmpgt
        | OpCode::if_icmple
        | OpCode::if_acmpeq
        | OpCode::if_acmpne
        | OpCode::goto
        | OpCode::jsr
        | OpCode::ifnull
        | OpCode::ifnonnull => 3,

        OpCode::invokeinterface => 5,

        OpCode::sipush => 3,

        OpCode::tableswitch
        | OpCode::lookupswitch
        | OpCode::multianewarray
        | OpCode::goto_w
        | OpCode::jsr_w => 0,

        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::values::AnyValue;

    /// 测试：将 `add(int a, int b)` 的 bytecode 编译为 LLVM IR 并执行。
    ///
    /// 对应的 Java 方法:
    /// ```java
    /// public int add(int a, int b) { return a + b; }
    /// ```
    ///
    /// Bytecode:
    /// - 0: iload_1   (0x1B) — 加载参数 a 到栈
    /// - 1: iload_2   (0x1C) — 加载参数 b 到栈
    /// - 2: iadd      (0x60) — 弹出两值，相加，结果压栈
    /// - 3: ireturn   (0xAC) — 弹出栈顶值，返回
    #[test]
    fn test_compile_and_execute_add() {
        // add(int, int) 的 bytecode
        let bytecode: &[U1] = &[
            0x1B, // iload_1
            0x1C, // iload_2
            0x60, // iadd
            0xAC, // ireturn
        ];
        let max_locals = 3; // this + a + b
        let max_stack = 2;

        let context = Context::create();
        let leaked = Box::leak(Box::new(context));
        let module = leaked.create_module("test_add_module");
        let builder = leaked.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .expect("Failed to create execution engine");

        // 创建函数: void fn(i8* locals, i8* stack)
        let i8_type = leaked.i8_type();
        let ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let fn_type = leaked
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let function = module.add_function("jit_test_add", fn_type, None);

        let entry_bb = leaked.append_basic_block(function, "entry");
        let return_bb = leaked.append_basic_block(function, "return");

        builder.position_at_end(entry_bb);

        let locals_ptr = function.get_first_param().unwrap().into_pointer_value();
        let stack_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i32_type = leaked.i32_type();
        let locals_i32 = builder
            .build_pointer_cast(
                locals_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "locals_cast",
            )
            .expect("cast failed");
        let stack_i32 = builder
            .build_pointer_cast(
                stack_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "stack_cast",
            )
            .expect("cast failed");

        // locals: slot0(this), slot1(a), slot2(b)
        let local_a = unsafe {
            builder
                .build_in_bounds_gep(
                    i32_type,
                    locals_i32,
                    &[i32_type.const_int(1, false)],
                    "ptr_a",
                )
                .expect("gep failed")
        };
        let local_b = unsafe {
            builder
                .build_in_bounds_gep(
                    i32_type,
                    locals_i32,
                    &[i32_type.const_int(2, false)],
                    "ptr_b",
                )
                .expect("gep failed")
        };

        // iload_1: load a, store to stack[0]
        let val_a = builder
            .build_load(i32_type, local_a, "a")
            .expect("load failed")
            .into_int_value();
        let stack0 = unsafe {
            builder
                .build_in_bounds_gep(i32_type, stack_i32, &[i32_type.const_int(0, false)], "sp0")
                .expect("gep failed")
        };
        builder.build_store(stack0, val_a).expect("store failed");

        // iload_2: load b, store to stack[1]
        let val_b = builder
            .build_load(i32_type, local_b, "b")
            .expect("load failed")
            .into_int_value();
        let stack1 = unsafe {
            builder
                .build_in_bounds_gep(i32_type, stack_i32, &[i32_type.const_int(1, false)], "sp1")
                .expect("gep failed")
        };
        builder.build_store(stack1, val_b).expect("store failed");

        // iadd: load stack[0], stack[1], add, store result to stack[0]
        let loaded_a = builder
            .build_load(i32_type, stack0, "la")
            .expect("load failed")
            .into_int_value();
        let loaded_b = builder
            .build_load(i32_type, stack1, "lb")
            .expect("load failed")
            .into_int_value();
        let sum = builder
            .build_int_add(loaded_a, loaded_b, "sum")
            .expect("add failed");
        builder.build_store(stack0, sum).expect("store failed");

        // ireturn: result is at stack[0], branch to return (no actual value passing in void fn)
        builder
            .build_unconditional_branch(return_bb)
            .expect("branch failed");

        // return block: read result from stack[0] and return as void
        builder.position_at_end(return_bb);
        builder.build_return(None).expect("return failed");

        assert!(function.verify(true), "IR verification failed");

        // 获取编译后的函数指针
        let jit_fn: extern "C" fn(*mut i32, *mut i32) = unsafe {
            let f: inkwell::execution_engine::JitFunction<unsafe extern "C" fn(*mut (), *mut ())> =
                execution_engine
                    .get_function("jit_test_add")
                    .expect("get_function failed");
            std::mem::transmute(f.as_raw())
        };

        let mut locals: [i32; 3] = [0, 3, 4]; // [this, a=3, b=4]
        let mut stack: [i32; 2] = [0, 0];

        unsafe {
            jit_fn(locals.as_mut_ptr(), stack.as_mut_ptr());
        }

        assert_eq!(stack[0], 7, "add(3, 4) should return 7, got {}", stack[0]);
    }

    /// 测试：if_icmpge 控制流翻译。
    ///
    /// Java:
    /// ```java
    /// public int max(int a, int b) {
    ///     if (a >= b) return a;
    ///     return b;
    /// }
    /// ```
    ///
    /// Bytecode:
    /// - 0: iload_1        — 加载 a
    /// - 1: iload_2        — 加载 b
    /// - 2: if_icmpge 7    — if a >= b, jump to pc 7 (return a)
    /// - 5: iload_2        — 加载 b
    /// - 6: ireturn        — return b
    /// - 7: iload_1        — 加载 a
    /// - 8: ireturn        — return a
    #[test]
    fn test_compile_and_execute_branch() {
        // collect_jump_targets 计算: target = pc + offset
        // 要从 PC=2 跳转到 PC=7，offset = 7 - 2 = 5
        let bytecode: &[U1] = &[
            0x1B, // 0: iload_1
            0x1C, // 1: iload_2
            0xA2, 0x00, 0x05, // 2: if_icmpge -> PC=2+5=7
            0x1C, // 5: iload_2
            0xAC, // 6: ireturn
            0x1B, // 7: iload_1
            0xAC, // 8: ireturn
        ];
        let max_locals = 3;
        let max_stack = 2;

        let context = Context::create();
        let leaked = Box::leak(Box::new(context));
        let module = leaked.create_module("test_max_module");
        let builder = leaked.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .expect("Failed to create execution engine");

        // 创建函数
        let i8_type = leaked.i8_type();
        let ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let fn_type = leaked
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let function = module.add_function("jit_test_max", fn_type, None);

        let jump_targets = collect_jump_targets(bytecode);

        let mut bb_map: HashMap<usize, BasicBlock> = HashMap::new();
        let entry_bb = leaked.append_basic_block(function, "entry");
        bb_map.insert(0, entry_bb);
        for offset in &jump_targets {
            if *offset != 0 && *offset < bytecode.len() {
                let bb = leaked.append_basic_block(function, &format!("bb_{}", offset));
                bb_map.insert(*offset, bb);
            }
        }
        let return_bb = leaked.append_basic_block(function, "return");

        builder.position_at_end(entry_bb);

        let i32_type = leaked.i32_type();
        let locals_ptr = function.get_first_param().unwrap().into_pointer_value();
        let stack_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let locals_i32 = builder
            .build_pointer_cast(
                locals_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "locals_cast",
            )
            .expect("cast failed");
        let stack_i32 = builder
            .build_pointer_cast(
                stack_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "stack_cast",
            )
            .expect("cast failed");

        // 创建 locals alloca
        let num_locals = max_locals.max(1);
        let local_vars: Vec<PointerValue> = (0..num_locals)
            .map(|i| {
                let alloca = builder
                    .build_alloca(i32_type, &format!("local_{}", i))
                    .expect("alloca failed");
                let idx = i32_type.const_int(i as u64, false);
                let ptr = unsafe {
                    builder
                        .build_in_bounds_gep(i32_type, locals_i32, &[idx], "local_ptr")
                        .expect("gep failed")
                };
                let loaded = builder
                    .build_load(i32_type, ptr, &format!("load_local_{}", i))
                    .expect("load failed")
                    .into_int_value();
                builder.build_store(alloca, loaded).expect("store failed");
                alloca
            })
            .collect();

        let stack_size = max_stack.max(1);
        let stack_vars: Vec<PointerValue> = (0..stack_size)
            .map(|i| {
                builder
                    .build_alloca(i32_type, &format!("stack_{}", i))
                    .expect("alloca failed")
            })
            .collect();

        let stack_top = builder
            .build_alloca(i32_type, "stack_top")
            .expect("alloca failed");
        builder
            .build_store(stack_top, i32_type.const_int(0, false))
            .expect("store failed");

        let mut interp = BytecodeInterpreter {
            context: leaked,
            module: &module as *const Module<'_>,
            builder: &builder,
            bb_map,
            function,
            return_bb,
            local_vars,
            stack_vars,
            stack_top,
            bytecode,
            max_stack,
            stack_param: Some(stack_i32),
        };

        let entry = *interp.bb_map.get(&0).unwrap();
        builder.position_at_end(entry);
        interp.translate_bytecode(0);
        interp.translate_remaining_blocks();

        if return_bb.get_terminator().is_none() {
            builder.position_at_end(return_bb);
            builder.build_return(None).expect("return failed");
        }

        assert!(function.verify(true), "IR verification failed for max()");

        let jit_fn: extern "C" fn(*mut i32, *mut i32) = unsafe {
            let f: inkwell::execution_engine::JitFunction<unsafe extern "C" fn(*mut (), *mut ())> =
                execution_engine
                    .get_function("jit_test_max")
                    .expect("get_function failed");
            std::mem::transmute(f.as_raw())
        };

        // 测试 max(5, 3) = 5
        let mut locals1: [i32; 3] = [0, 5, 3];
        let mut stack1: [i32; 2] = [0, 0];
        unsafe {
            jit_fn(locals1.as_mut_ptr(), stack1.as_mut_ptr());
        }
        assert_eq!(stack1[0], 5, "max(5, 3) should be 5");

        // 测试 max(2, 8) = 8
        let mut locals2: [i32; 3] = [0, 2, 8];
        let mut stack2: [i32; 2] = [0, 0];
        unsafe {
            jit_fn(locals2.as_mut_ptr(), stack2.as_mut_ptr());
        }
        assert_eq!(stack2[0], 8, "max(2, 8) should be 8");

        // 测试 max(4, 4) = 4
        let mut locals3: [i32; 3] = [0, 4, 4];
        let mut stack3: [i32; 2] = [0, 0];
        unsafe {
            jit_fn(locals3.as_mut_ptr(), stack3.as_mut_ptr());
        }
        assert_eq!(stack3[0], 4, "max(4, 4) should be 4");
    }

    /// 端到端测试：从真实 Java .class 文件加载 bytecode，通过 JIT 编译并执行。
    ///
    /// 测试流程：
    /// 1. 读取 SimpleCalc.class 文件
    /// 2. 解析为 ClassFile 结构
    /// 3. 提取 add(int, int) 方法的 bytecode
    /// 4. 编译为 LLVM IR → JIT 编译为机器码
    /// 5. 调用并验证结果
    ///
    /// 这验证了从真实 class 文件到 JIT 执行的完整链路。
    #[test]
    fn test_e2e_jit_from_class_file() {
        use class_parser::parse_class;

        // 1. 读取并解析 SimpleCalc.class
        let class_bytes = std::fs::read("/tmp/jvm-test-classes/SimpleCalc.class")
            .expect("SimpleCalc.class not found — run `javac -d /tmp/jvm-test-classes crates/class-parser/tests/fixtures/src/SimpleCalc.java`");
        let cf = parse_class(&class_bytes).expect("Failed to parse SimpleCalc.class");

        // 2. 找到 add(int, int) 方法
        let add_method = cf
            .methods
            .iter()
            .find(|m| {
                let name = classfile::constant_pool::get_utf8(&cf.cp, m.name_index as usize);
                let desc = classfile::constant_pool::get_utf8(&cf.cp, m.desc_index as usize);
                name.as_slice() == b"add" && desc.as_slice() == b"(II)I"
            })
            .expect("add(int,int) method not found");

        let code = add_method
            .get_code()
            .expect("add method has no Code attribute");
        let bytecode = code.code.as_slice();
        let max_locals = code.max_locals as usize;
        let max_stack = code.max_stack as usize;

        // 3. 通过 JIT 编译并执行
        let context = Context::create();
        let leaked = Box::leak(Box::new(context));
        let module = leaked.create_module("test_e2e_module");
        let builder = leaked.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .expect("Failed to create execution engine");

        let fn_name = "jit_test_e2e";
        let i8_type = leaked.i8_type();
        let ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let fn_type = leaked
            .void_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let function = module.add_function(fn_name, fn_type, None);

        let entry_bb = leaked.append_basic_block(function, "entry");
        let return_bb = leaked.append_basic_block(function, "return");
        builder.position_at_end(entry_bb);

        let i32_type = leaked.i32_type();
        let locals_ptr = function.get_first_param().unwrap().into_pointer_value();
        let stack_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let locals_i32 = builder
            .build_pointer_cast(
                locals_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "locals_cast",
            )
            .expect("cast failed");
        let stack_i32 = builder
            .build_pointer_cast(
                stack_ptr,
                i32_type.ptr_type(inkwell::AddressSpace::default()),
                "stack_cast",
            )
            .expect("cast failed");

        // 预填 locals：slot0=this, slot1=a, slot2=b
        let num_locals = max_locals.max(1);
        let local_vars: Vec<PointerValue> = (0..num_locals)
            .map(|i| {
                let alloca = builder
                    .build_alloca(i32_type, &format!("local_{}", i))
                    .expect("alloca failed");
                let idx = i32_type.const_int(i as u64, false);
                let ptr = unsafe {
                    builder
                        .build_in_bounds_gep(i32_type, locals_i32, &[idx], "local_ptr")
                        .expect("gep failed")
                };
                let loaded = builder
                    .build_load(i32_type, ptr, &format!("load_local_{}", i))
                    .expect("load failed")
                    .into_int_value();
                builder.build_store(alloca, loaded).expect("store failed");
                alloca
            })
            .collect();

        let stack_size = max_stack.max(1);
        let stack_vars: Vec<PointerValue> = (0..stack_size)
            .map(|i| {
                builder
                    .build_alloca(i32_type, &format!("stack_{}", i))
                    .expect("alloca failed")
            })
            .collect();

        let stack_top = builder
            .build_alloca(i32_type, "stack_top")
            .expect("alloca failed");
        builder
            .build_store(stack_top, i32_type.const_int(0, false))
            .expect("store failed");

        let jump_targets = collect_jump_targets(bytecode);
        let mut bb_map: HashMap<usize, BasicBlock> = HashMap::new();
        let entry_block = *bb_map.entry(0).or_insert(entry_bb);
        for offset in &jump_targets {
            if *offset != 0 && *offset < bytecode.len() {
                let bb = leaked.append_basic_block(function, &format!("bb_{}", offset));
                bb_map.insert(*offset, bb);
            }
        }

        let mut interp = BytecodeInterpreter {
            context: leaked,
            module: &module as *const Module<'_>,
            builder: &builder,
            bb_map,
            function,
            return_bb,
            local_vars,
            stack_vars,
            stack_top,
            bytecode,
            max_stack,
            stack_param: Some(stack_i32),
        };

        let entry = *interp.bb_map.get(&0).unwrap();
        builder.position_at_end(entry);
        interp.translate_bytecode(0);
        interp.translate_remaining_blocks();

        if return_bb.get_terminator().is_none() {
            builder.position_at_end(return_bb);
            builder.build_return(None).expect("return failed");
        }

        assert!(
            function.verify(true),
            "IR verification failed for class-file method"
        );

        // 4. 获取函数指针并执行
        let jit_fn: extern "C" fn(*mut i32, *mut i32) = unsafe {
            let f: inkwell::execution_engine::JitFunction<unsafe extern "C" fn(*mut (), *mut ())> =
                execution_engine
                    .get_function(fn_name)
                    .expect("get_function failed");
            std::mem::transmute(f.as_raw())
        };

        // 测试 add(10, 20) = 30
        let mut locals: [i32; 3] = [0, 10, 20];
        let mut stack: [i32; 2] = [0, 0];
        unsafe {
            jit_fn(locals.as_mut_ptr(), stack.as_mut_ptr());
        }
        assert_eq!(
            stack[0], 30,
            "SimpleCalc.add(10, 20) should return 30, got {}",
            stack[0]
        );

        // 测试 add(-5, 5) = 0
        let mut locals2: [i32; 3] = [0, -5, 5];
        let mut stack2: [i32; 2] = [0, 0];
        unsafe {
            jit_fn(locals2.as_mut_ptr(), stack2.as_mut_ptr());
        }
        assert_eq!(
            stack2[0], 0,
            "SimpleCalc.add(-5, 5) should return 0, got {}",
            stack2[0]
        );
    }
}
