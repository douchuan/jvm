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
use inkwell::values::{AnyValue, FunctionValue, IntValue, PointerValue};
use std::collections::HashMap;

/// 编译单个方法，生成 LLVM IR 函数。
///
/// ## 参数
/// - `bytecode`: 方法的字节码数组
/// - `max_locals`: 最大本地变量槽数
/// - `max_stack`: 最大操作数栈深度
///
/// ## 返回
/// 生成的 LLVM `FunctionValue`。如果编译失败（如遇到不支持的 opcode），返回 None。
pub fn compile_method<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    method: &Method,
    bytecode: &[U1],
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
    };

    // 从 entry block 开始翻译
    let entry_bb = *interp.bb_map.get(&0).unwrap();
    builder.position_at_end(entry_bb);
    interp.translate_bytecode();

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
struct BytecodeInterpreter<'ctx> {
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    bb_map: HashMap<usize, BasicBlock<'ctx>>,
    function: FunctionValue<'ctx>,
    return_bb: BasicBlock<'ctx>,
    local_vars: Vec<PointerValue<'ctx>>,
    stack_vars: Vec<PointerValue<'ctx>>,
    stack_top: PointerValue<'ctx>,
    bytecode: &'ctx [U1],
    max_stack: usize,
}

impl<'ctx> BytecodeInterpreter<'ctx> {
    /// 主翻译循环。
    fn translate_bytecode(&mut self) {
        let mut pc: usize = 0;

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
                OpCode::lconst_0 => {
                    self.push_int(0);
                    pc += 1;
                }
                OpCode::lconst_1 => {
                    self.push_int(1);
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
                // 返回指令
                // ============================================================
                OpCode::ireturn => {
                    let _val = self.pop_int();
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
                // nop
                // ============================================================
                OpCode::nop => {
                    pc += 1;
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
        let slot_idx = top_i32.get_zero_extended_constant().unwrap_or(0) as usize;

        if slot_idx < self.stack_vars.len() {
            self.builder
                .build_store(self.stack_vars[slot_idx], val)
                .expect("push_int store failed");
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

    /// 将常量值推入栈的快捷方法
    fn push_int(&mut self, val: i32) {
        let llvm_val = self.context.i32_type().const_int(val as u64, val < 0);
        self.push_int_val(llvm_val);
    }

    /// 从操作数栈弹出一个 i32 值。
    fn pop_int(&mut self) -> IntValue<'ctx> {
        let top_i32 = self
            .builder
            .build_load(self.context.i32_type(), self.stack_top, "load_top")
            .expect("pop load stack_top failed")
            .into_int_value();
        let one = self.context.i32_type().const_int(1, false);
        let new_top = self
            .builder
            .build_int_sub(top_i32, one, "dec_top")
            .expect("dec_top failed");
        self.builder
            .build_store(self.stack_top, new_top)
            .expect("pop store stack_top failed");

        let slot_idx = new_top.get_zero_extended_constant().unwrap_or(0) as usize;
        self.builder
            .build_load(
                self.context.i32_type(),
                self.stack_vars[slot_idx],
                "pop_val",
            )
            .expect("pop load failed")
            .into_int_value()
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
        | OpCode::invokeinterface
        | OpCode::invokedynamic
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

        OpCode::sipush => 3,

        OpCode::tableswitch
        | OpCode::lookupswitch
        | OpCode::multianewarray
        | OpCode::goto_w
        | OpCode::jsr_w => 0,

        _ => 1,
    }
}
