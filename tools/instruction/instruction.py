# -*- coding: utf-8 -*-
import sys

# (name, 1 + encoded_value_length, bytecode value)
# encoded_value to calc index in constant pool
instructions = [
    ("aaload", 1, 50),
    ("aastore", 1, 83),
    ("aconst_null", 1, 1),
    ("aload", 2, 25),
    ("aload_0", 1, 42),
    ("aload_1", 1, 43),
    ("aload_2", 1, 44),
    ("aload_3", 1, 45),
    ("anewarray", 3, 189),
    ("areturn", 1, 176),
    ("arraylength", 1, 190),
    ("astore", 2, 58),
    ("astore_0", 1, 75),
    ("astore_1", 1, 76),
    ("astore_2", 1, 77),
    ("astore_3", 1, 78),
    ("athrow", 1, 191),
    ("baload", 1, 51),
    ("bastore", 1, 84),
    ("bipush", 2, 16),
    ("caload", 1, 52),
    ("castore", 1, 85),
    ("checkcast", 3, 192),
    ("d2f", 1, 144),
    ("d2i", 1, 142),
    ("d2l", 1, 143),
    ("dadd", 1, 99),
    ("daload", 2, 49),
    ("dastore", 1, 82),
    ("dcmpg", 1, 152),
    ("dcmpl", 1, 151),
    ("dconst_0", 1, 14),
    ("dconst_1", 1, 15),
    ("ddiv", 1, 111),
    ("dload", 2, 24),
    ("dload_0", 1, 38),
    ("dload_1", 1, 39),
    ("dload_2", 1, 40),
    ("dload_3", 1, 41),
    ("dmul", 1, 107),
    ("dneg", 1, 119),
    ("drem", 1, 115),
    ("dreturn", 1, 175),
    ("dstore", 2, 57),
    ("dstore_0", 1, 71),
    ("dstore_1", 1, 72),
    ("dstore_2", 1, 73),
    ("dstore_3", 1, 74),
    ("dsub", 1, 103),
    ("dup", 1, 89),
    ("dup2", 1, 92),
    ("dup2_x1", 1, 93),
    ("dup2_x2", 1, 94),
    ("dup_x1", 1, 90),
    ("dup_x2", 1, 91),
    ("f2d", 1, 141),
    ("f2i", 1, 139),
    ("f2l", 1, 140),
    ("fadd", 1, 98),
    ("faload", 1, 48),
    ("fastore", 1, 81),
    ("fcmpg", 1, 150),
    ("fcmpl", 1, 149),
    ("fconst_0", 1, 11),
    ("fconst_1", 1, 12),
    ("fconst_2", 1, 13),
    ("fdiv", 1, 110),
    ("fload", 2, 23),
    ("fload_0", 1, 34),
    ("fload_1", 1, 35),
    ("fload_2", 1, 36),
    ("fload_3", 1, 37),
    ("fmul", 1, 106),
    ("fneg", 1, 118),
    ("frem", 1, 114),
    ("freturn", 1, 174),
    ("fstore", 2, 56),
    ("fstore_0", 1, 67),
    ("fstore_1", 1, 68),
    ("fstore_2", 1, 69),
    ("fstore_3", 1, 70),
    ("fsub", 1, 102),
    ("getfield", 3, 180),
    ("getstatic", 3, 178),
    ("goto", 3, 167),
    ("goto_w", 5, 200),
    ("i2b", 1, 145),
    ("i2c", 1, 146),
    ("i2d", 1, 135),
    ("i2f", 1, 134),
    ("i2l", 1, 133),
    ("i2s", 1, 147),
    ("iadd", 1, 96),
    ("iaload", 1, 46),
    ("iand", 1, 126),
    ("iastore", 1, 79),
    ("iconst_0", 1, 3),
    ("iconst_1", 1, 4),
    ("iconst_2", 1, 5),
    ("iconst_3", 1, 6),
("iconst_4", 1, 7),
("iconst_5", 1, 8),
("iconst_m1", 1, 2),
("idiv", 1, 108),
("if_acmpeq", 3, 165),
("if_acmpne", 3, 166),
("if_icmpeq", 3, 159),
("if_icmpge", 3, 162),
("if_icmpgt", 3, 163),
("if_icmple", 3, 164),
("if_icmplt", 3, 161),
("if_icmpne", 3, 160),
("ifeq", 3, 153),
("ifge", 3, 156),
("ifgt", 3, 157),
("ifle", 3, 158),
("iflt", 3, 155),
("ifne", 3, 154),
("ifnonnull", 3, 199),
("ifnull", 3, 198),
("iinc", 3, 132),
("iload", 2, 21),
("iload_0", 1, 26),
("iload_1", 1, 27),
("iload_2", 1, 28),
("iload_3", 1, 29),
("imul", 1, 104),
("ineg", 1, 116),
("instanceof", 3, 193),
("invokedynamic", 5, 186),
("invokeinterface", 5, 185),
("invokespecial", 3, 183),
("invokestatic", 3, 184),
("invokevirtual", 3, 182),
("ior", 1, 128),
("irem", 1, 112),
("ireturn", 1, 172),
("ishl", 1, 120),
("ishr", 1, 122),
("istore", 2, 54),
("istore_0", 1, 59),
("istore_1", 1, 60),
("istore_2", 1, 61),
("istore_3", 1, 62),
("isub", 1, 100),
("iushr", 1, 124),
("ixor", 1, 130),
("jsr", 3, 168),
("jsr_w", 5, 201),
("l2d", 1, 138),
("l2f", 1, 137),
("l2i", 1, 136),
("ladd", 1, 97),
("laload", 1, 47),
("land", 1, 127),
("lastore", 1, 80),
("lcmp", 1, 148),
("lconst_0", 1, 9),
("lconst_1", 1, 10),
("ldc", 2, 18),
("ldc2_w", 3, 20),
("ldc_w", 3, 19),
("ldiv", 1, 109),
("lload", 2, 22),
("lload_0", 1, 30),
("lload_1", 1, 31),
("lload_2", 1, 32),
("lload_3", 1, 33),
("lmul", 1, 105),
("lneg", 1, 117),
("lookupswitch", "variable-length instruction", 171),
("lor", 1, 129),
("lrem", 1, 113),
("lreturn", 1, 173),
("lshl", 1, 121),
("lshr", 1, 123),
("lstore", 2, 55),
("lstore_0", 1, 63),
("lstore_1", 1, 64),
("lstore_2", 1, 65),
("lstore_3", 1, 66),
("lsub", 1, 101),
("lushr", 1, 125),
("lxor", 1, 131),
("monitorenter", 1, 194),
("monitorexit", 1, 195),
("multianewarray", 4, 197),
("new", 3, 187),
("newarray", 2, 188),
("nop", 1, 0),
("pop", 1, 87),
("pop2", 1, 88),
("putfield", 3, 181),
("putstatic", 3, 179),
("ret", 2, 169),
("return_void", 1, 177),
("saload", 1, 53),
("sastore", 1, 86),
("sipush", 3, 17),
("swap", 1, 95),
("tableswitch", "variable-length instruction", 170),
("wide", 1, 196),
]

def create_get_instructions(ary):
    ary = sorted(ary, key=lambda it: it[2])
    with open("mod.get_instructions", "w") as f:
        f.write("pub fn get_instructions() -> Vec<Box<dyn Instruction>> {\n")
        f.write("\tvec![\n")
        for it in ary:
            name = it[0]
            f.write("\t\tBox::new(" + name.title() + "),\n")
        f.write("\t]\n")
        f.write("}\n")


def create_uses(ary):
    with open("mod.uses", "w") as f:
        for it in ary:
            name = it[0]
            f.write("mod " + name + ";\n")
        f.write("\n")
        for it in ary:
            name = it[0]
            f.write("use " + name + "::" + name.title() + ";\n")

def create_mod(instruction):
    (name, step, _) = instruction
    with open(name + ".rs", "w") as f:
        f.write("#![allow(non_camel_case_types)]\n")
        f.write("use classfile::OpCode;\n")
        f.write("use super::{Instruction, InstructionInfo};\n")
        f.write("\n")
        f.write("pub struct " + name.title() + ";\n")
        f.write("\n")
        f.write("impl Instruction for " + name.title() + " {\n")
        f.write("   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {\n")
        f.write("       let info = InstructionInfo {\n")
        f.write("           op_code: OpCode::" + name + ",\n")
        if name in ["instanceof", "multianewarray", "invokevirtual", "anewarray", "checkcast", 
            "putfield", "getfield", "getstatic", "invokespecial", "ldc2_w", "invokeinterface", 
            "new", "invokestatic", "ldc_w", "putstatic", "invokedynamic"]:
            f.write("           icp: self.calc_cp_index_u16(codes, pc)\n")
        else:
            f.write("           icp: 0\n")
        f.write("       };\n")
        f.write("\n")
        if name in ["lookupswitch", "tableswitch", "wide"]:
            f.write("\tunimplemented!(\"" + str(step) + "\")")
        else:
            f.write("       (info, pc + " + str(step) + ")\n")
        f.write("   }\n")
        f.write("}")

if __name__ == '__main__':
    # create_get_instructions(instructions)
    # create_uses(instructions)
    for it in instructions:
        create_mod(it)



