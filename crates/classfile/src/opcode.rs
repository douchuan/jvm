#![allow(non_camel_case_types)]

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OpCode {
    //Constants
    nop,
    aconst_null,
    iconst_m1,
    iconst_0,
    iconst_1,
    iconst_2,
    iconst_3,
    iconst_4,
    iconst_5,
    lconst_0,
    lconst_1,
    fconst_0,
    fconst_1,
    fconst_2,
    dconst_0,
    dconst_1,
    bipush,
    sipush,
    ldc,
    ldc_w,
    ldc2_w,
    //Loads
    iload,
    lload,
    fload,
    dload,
    aload,
    iload_0,
    iload_1,
    iload_2,
    iload_3,
    lload_0,
    lload_1,
    lload_2,
    lload_3,
    fload_0,
    fload_1,
    fload_2,
    fload_3,
    dload_0,
    dload_1,
    dload_2,
    dload_3,
    aload_0,
    aload_1,
    aload_2,
    aload_3,
    iaload,
    laload,
    faload,
    daload,
    aaload,
    baload,
    caload,
    saload,
    //Stores
    istore,
    lstore,
    fstore,
    dstore,
    astore,
    istore_0,
    istore_1,
    istore_2,
    istore_3,
    lstore_0,
    lstore_1,
    lstore_2,
    lstore_3,
    fstore_0,
    fstore_1,
    fstore_2,
    fstore_3,
    dstore_0,
    dstore_1,
    dstore_2,
    dstore_3,
    astore_0,
    astore_1,
    astore_2,
    astore_3,
    iastore,
    lastore,
    fastore,
    dastore,
    aastore,
    bastore,
    castore,
    sastore,
    //Stack
    pop,
    pop2,
    dup,
    dup_x1,
    dup_x2,
    dup2,
    dup2_x1,
    dup2_x2,
    swap,
    //Math
    iadd,
    ladd,
    fadd,
    dadd,
    isub,
    lsub,
    fsub,
    dsub,
    imul,
    lmul,
    fmul,
    dmul,
    idiv,
    ldiv,
    fdiv,
    ddiv,
    irem,
    lrem,
    frem, //deprecated
    drem, //deprecated
    ineg,
    lneg,
    fneg, //deprecated
    dneg, //deprecated
    ishl,
    lshl,
    ishr,
    lshr,
    iushr,
    lushr,
    iand,
    land,
    ior,
    lor,
    ixor,
    lxor,
    iinc,
    //Conversions
    i2l,
    i2f,
    i2d,
    l2i,
    l2f,
    l2d,
    f2i,
    f2l,
    f2d,
    d2i,
    d2l,
    d2f,
    i2b,
    i2c,
    i2s,
    //Comparisons
    lcmp,
    fcmpl,
    fcmpg,
    dcmpl,
    dcmpg,
    ifeq,
    ifne,
    iflt,
    ifge,
    ifgt,
    ifle,
    if_icmpeq,
    if_icmpne,
    if_icmplt,
    if_icmpge,
    if_icmpgt,
    if_icmple,
    if_acmpeq,
    if_acmpne,
    //Control
    goto,
    jsr, //deprecated
    ret, //deprecated
    tableswitch,
    lookupswitch,
    ireturn,
    lreturn,
    freturn,
    dreturn,
    areturn,
    return_void,
    //References
    getstatic,
    putstatic,
    getfield,
    putfield,
    invokevirtual,
    invokespecial,
    invokestatic,
    invokeinterface,
    invokedynamic,
    new,
    newarray,
    anewarray,
    arraylength,
    athrow,
    checkcast,
    instanceof,
    monitorenter,
    monitorexit,
    //Extended
    wide, //deprecated
    multianewarray,
    ifnull,
    ifnonnull,
    goto_w, //deprecated
    jsr_w,  //deprecated
    reserved,
    breakpoint,
    impdep1,
    impdep2,
}

impl From<u8> for OpCode {
    #[inline]
    fn from(v: u8) -> Self {
        match v {
            0 => OpCode::nop,
            1 => OpCode::aconst_null,
            2 => OpCode::iconst_m1,
            3 => OpCode::iconst_0,
            4 => OpCode::iconst_1,
            5 => OpCode::iconst_2,
            6 => OpCode::iconst_3,
            7 => OpCode::iconst_4,
            8 => OpCode::iconst_5,
            9 => OpCode::lconst_0,
            10 => OpCode::lconst_1,
            11 => OpCode::fconst_0,
            12 => OpCode::fconst_1,
            13 => OpCode::fconst_2,
            14 => OpCode::dconst_0,
            15 => OpCode::dconst_1,
            16 => OpCode::bipush,
            17 => OpCode::sipush,
            18 => OpCode::ldc,
            19 => OpCode::ldc_w,
            20 => OpCode::ldc2_w,
            21 => OpCode::iload,
            22 => OpCode::lload,
            23 => OpCode::fload,
            24 => OpCode::dload,
            25 => OpCode::aload,
            26 => OpCode::iload_0,
            27 => OpCode::iload_1,
            28 => OpCode::iload_2,
            29 => OpCode::iload_3,
            30 => OpCode::lload_0,
            31 => OpCode::lload_1,
            32 => OpCode::lload_2,
            33 => OpCode::lload_3,
            34 => OpCode::fload_0,
            35 => OpCode::fload_1,
            36 => OpCode::fload_2,
            37 => OpCode::fload_3,
            38 => OpCode::dload_0,
            39 => OpCode::dload_1,
            40 => OpCode::dload_2,
            41 => OpCode::dload_3,
            42 => OpCode::aload_0,
            43 => OpCode::aload_1,
            44 => OpCode::aload_2,
            45 => OpCode::aload_3,
            46 => OpCode::iaload,
            47 => OpCode::laload,
            48 => OpCode::faload,
            49 => OpCode::daload,
            50 => OpCode::aaload,
            51 => OpCode::baload,
            52 => OpCode::caload,
            53 => OpCode::saload,
            54 => OpCode::istore,
            55 => OpCode::lstore,
            56 => OpCode::fstore,
            57 => OpCode::dstore,
            58 => OpCode::astore,
            59 => OpCode::istore_0,
            60 => OpCode::istore_1,
            61 => OpCode::istore_2,
            62 => OpCode::istore_3,
            63 => OpCode::lstore_0,
            64 => OpCode::lstore_1,
            65 => OpCode::lstore_2,
            66 => OpCode::lstore_3,
            67 => OpCode::fstore_0,
            68 => OpCode::fstore_1,
            69 => OpCode::fstore_2,
            70 => OpCode::fstore_3,
            71 => OpCode::dstore_0,
            72 => OpCode::dstore_1,
            73 => OpCode::dstore_2,
            74 => OpCode::dstore_3,
            75 => OpCode::astore_0,
            76 => OpCode::astore_1,
            77 => OpCode::astore_2,
            78 => OpCode::astore_3,
            79 => OpCode::iastore,
            80 => OpCode::lastore,
            81 => OpCode::fastore,
            82 => OpCode::dastore,
            83 => OpCode::aastore,
            84 => OpCode::bastore,
            85 => OpCode::castore,
            86 => OpCode::sastore,
            87 => OpCode::pop,
            88 => OpCode::pop2,
            89 => OpCode::dup,
            90 => OpCode::dup_x1,
            91 => OpCode::dup_x2,
            92 => OpCode::dup2,
            93 => OpCode::dup2_x1,
            94 => OpCode::dup_x2,
            95 => OpCode::swap,
            96 => OpCode::iadd,
            97 => OpCode::ladd,
            98 => OpCode::fadd,
            99 => OpCode::dadd,
            100 => OpCode::isub,
            101 => OpCode::lsub,
            102 => OpCode::fsub,
            103 => OpCode::dsub,
            104 => OpCode::imul,
            105 => OpCode::lmul,
            106 => OpCode::fmul,
            107 => OpCode::dmul,
            108 => OpCode::idiv,
            109 => OpCode::ldiv,
            110 => OpCode::fdiv,
            111 => OpCode::ddiv,
            112 => OpCode::irem,
            113 => OpCode::lrem,
            114 => OpCode::frem,
            115 => OpCode::drem,
            116 => OpCode::ineg,
            117 => OpCode::lneg,
            118 => OpCode::fneg,
            119 => OpCode::dneg,
            120 => OpCode::ishl,
            121 => OpCode::lshl,
            122 => OpCode::ishr,
            123 => OpCode::lshr,
            124 => OpCode::iushr,
            125 => OpCode::lushr,
            126 => OpCode::iand,
            127 => OpCode::land,
            128 => OpCode::ior,
            129 => OpCode::lor,
            130 => OpCode::ixor,
            131 => OpCode::lxor,
            132 => OpCode::iinc,
            133 => OpCode::i2l,
            134 => OpCode::i2f,
            135 => OpCode::i2d,
            136 => OpCode::l2i,
            137 => OpCode::l2f,
            138 => OpCode::l2d,
            139 => OpCode::f2i,
            140 => OpCode::f2l,
            141 => OpCode::f2d,
            142 => OpCode::d2i,
            143 => OpCode::d2l,
            144 => OpCode::d2f,
            145 => OpCode::i2b,
            146 => OpCode::i2c,
            147 => OpCode::i2s,
            148 => OpCode::lcmp,
            149 => OpCode::fcmpl,
            150 => OpCode::fcmpg,
            151 => OpCode::dcmpl,
            152 => OpCode::dcmpg,
            153 => OpCode::ifeq,
            154 => OpCode::ifne,
            155 => OpCode::iflt,
            156 => OpCode::ifge,
            157 => OpCode::ifgt,
            158 => OpCode::ifle,
            159 => OpCode::if_icmpeq,
            160 => OpCode::if_icmpne,
            161 => OpCode::if_icmplt,
            162 => OpCode::if_icmpge,
            163 => OpCode::if_icmpgt,
            164 => OpCode::if_icmple,
            165 => OpCode::if_acmpeq,
            166 => OpCode::if_acmpne,
            167 => OpCode::goto,
            168 => OpCode::jsr,
            169 => OpCode::ret,
            170 => OpCode::tableswitch,
            171 => OpCode::lookupswitch,
            172 => OpCode::ireturn,
            173 => OpCode::lreturn,
            174 => OpCode::freturn,
            175 => OpCode::dreturn,
            176 => OpCode::areturn,
            177 => OpCode::return_void,
            178 => OpCode::getstatic,
            179 => OpCode::putstatic,
            180 => OpCode::getfield,
            181 => OpCode::putfield,
            182 => OpCode::invokevirtual,
            183 => OpCode::invokespecial,
            184 => OpCode::invokestatic,
            185 => OpCode::invokeinterface,
            186 => OpCode::invokedynamic,
            187 => OpCode::new,
            188 => OpCode::newarray,
            189 => OpCode::anewarray,
            190 => OpCode::arraylength,
            191 => OpCode::athrow,
            192 => OpCode::checkcast,
            193 => OpCode::instanceof,
            194 => OpCode::monitorenter,
            195 => OpCode::monitorexit,
            196 => OpCode::wide,
            197 => OpCode::multianewarray,
            198 => OpCode::ifnull,
            199 => OpCode::ifnonnull,
            200 => OpCode::goto_w,
            201 => OpCode::jsr_w,
            202 => OpCode::breakpoint,
            254 => OpCode::impdep1,
            255 => OpCode::impdep2,
            _ => OpCode::reserved,
        }
    }
}

impl Into<&'static str> for OpCode {
    fn into(self) -> &'static str {
        match self {
            OpCode::nop => "nop",
            OpCode::aconst_null => "aconst_null",
            OpCode::iconst_m1 => "iconst_m1",
            OpCode::iconst_0 => "iconst_0",
            OpCode::iconst_1 => "iconst_1",
            OpCode::iconst_2 => "iconst_2",
            OpCode::iconst_3 => "iconst_3",
            OpCode::iconst_4 => "iconst_4",
            OpCode::iconst_5 => "iconst_5",
            OpCode::lconst_0 => "lconst_0",
            OpCode::lconst_1 => "lconst_1",
            OpCode::fconst_0 => "fconst_0",
            OpCode::fconst_1 => "fconst_1",
            OpCode::fconst_2 => "fconst_2",
            OpCode::dconst_0 => "dconst_0",
            OpCode::dconst_1 => "dconst_1",
            OpCode::bipush => "bipush",
            OpCode::sipush => "sipush",
            OpCode::ldc => "ldc",
            OpCode::ldc_w => "ldc_w",
            OpCode::ldc2_w => "ldc2_w",
            OpCode::iload => "iload",
            OpCode::lload => "lload",
            OpCode::fload => "fload",
            OpCode::dload => "dload",
            OpCode::aload => "aload",
            OpCode::iload_0 => "iload_0",
            OpCode::iload_1 => "iload_1",
            OpCode::iload_2 => "iload_2",
            OpCode::iload_3 => "iload_3",
            OpCode::lload_0 => "lload_0",
            OpCode::lload_1 => "lload_1",
            OpCode::lload_2 => "lload_2",
            OpCode::lload_3 => "lload_3",
            OpCode::fload_0 => "fload_0",
            OpCode::fload_1 => "fload_1",
            OpCode::fload_2 => "fload_2",
            OpCode::fload_3 => "fload_3",
            OpCode::dload_0 => "dload_0",
            OpCode::dload_1 => "dload_1",
            OpCode::dload_2 => "dload_2",
            OpCode::dload_3 => "dload_3",
            OpCode::aload_0 => "aload_0",
            OpCode::aload_1 => "aload_1",
            OpCode::aload_2 => "aload_2",
            OpCode::aload_3 => "aload_3",
            OpCode::iaload => "iaload",
            OpCode::laload => "laload",
            OpCode::faload => "faload",
            OpCode::daload => "daload",
            OpCode::aaload => "aaload",
            OpCode::baload => "baload",
            OpCode::caload => "caload",
            OpCode::saload => "saload",
            OpCode::istore => "istore",
            OpCode::lstore => "lstore",
            OpCode::fstore => "fstore",
            OpCode::dstore => "dstore",
            OpCode::astore => "astore",
            OpCode::istore_0 => "istore_0",
            OpCode::istore_1 => "istore_1",
            OpCode::istore_2 => "istore_2",
            OpCode::istore_3 => "istore_3",
            OpCode::lstore_0 => "lstore_0",
            OpCode::lstore_1 => "lstore_1",
            OpCode::lstore_2 => "lstore_2",
            OpCode::lstore_3 => "lstore_3",
            OpCode::fstore_0 => "fstore_0",
            OpCode::fstore_1 => "fstore_1",
            OpCode::fstore_2 => "fstore_2",
            OpCode::fstore_3 => "fstore_3",
            OpCode::dstore_0 => "dstore_0",
            OpCode::dstore_1 => "dstore_1",
            OpCode::dstore_2 => "dstore_2",
            OpCode::dstore_3 => "dstore_3",
            OpCode::astore_0 => "astore_0",
            OpCode::astore_1 => "astore_1",
            OpCode::astore_2 => "astore_2",
            OpCode::astore_3 => "astore_3",
            OpCode::iastore => "iastore",
            OpCode::lastore => "lastore",
            OpCode::fastore => "fastore",
            OpCode::dastore => "dastore",
            OpCode::aastore => "aastore",
            OpCode::bastore => "bastore",
            OpCode::castore => "castore",
            OpCode::sastore => "sastore",
            OpCode::pop => "pop",
            OpCode::pop2 => "pop2",
            OpCode::dup => "dup",
            OpCode::dup_x1 => "dup_x1",
            OpCode::dup_x2 => "dup_x2",
            OpCode::dup2 => "dup2",
            OpCode::dup2_x1 => "dup2_x1",
            OpCode::dup2_x2 => "dup2_x2",
            OpCode::swap => "swap",
            OpCode::iadd => "iadd",
            OpCode::ladd => "ladd",
            OpCode::fadd => "fadd",
            OpCode::dadd => "dadd",
            OpCode::isub => "isub",
            OpCode::lsub => "lsub",
            OpCode::fsub => "fsub",
            OpCode::dsub => "dsub",
            OpCode::imul => "imul",
            OpCode::lmul => "lmul",
            OpCode::fmul => "fmul",
            OpCode::dmul => "dmul",
            OpCode::idiv => "idiv",
            OpCode::ldiv => "ldiv",
            OpCode::fdiv => "fdiv",
            OpCode::ddiv => "ddiv",
            OpCode::irem => "irem",
            OpCode::lrem => "lrem",
            OpCode::frem => "frem",
            OpCode::drem => "drem",
            OpCode::ineg => "ineg",
            OpCode::lneg => "lneg",
            OpCode::fneg => "fneg",
            OpCode::dneg => "dneg",
            OpCode::ishl => "ishl",
            OpCode::lshl => "lshl",
            OpCode::ishr => "ishr",
            OpCode::lshr => "lshr",
            OpCode::iushr => "iushr",
            OpCode::lushr => "lushr",
            OpCode::iand => "iand",
            OpCode::land => "land",
            OpCode::ior => "ior",
            OpCode::lor => "lor",
            OpCode::ixor => "ixor",
            OpCode::lxor => "lxor",
            OpCode::iinc => "iinc",
            OpCode::i2l => "i2l",
            OpCode::i2f => "i2f",
            OpCode::i2d => "i2d",
            OpCode::l2i => "l2i",
            OpCode::l2f => "l2f",
            OpCode::l2d => "l2d",
            OpCode::f2i => "f2i",
            OpCode::f2l => "f2l",
            OpCode::f2d => "f2d",
            OpCode::d2i => "d2i",
            OpCode::d2l => "d2l",
            OpCode::d2f => "d2f",
            OpCode::i2b => "i2b",
            OpCode::i2c => "i2c",
            OpCode::i2s => "i2s",
            OpCode::lcmp => "lcmp",
            OpCode::fcmpl => "fcmpl",
            OpCode::fcmpg => "fcmpg",
            OpCode::dcmpl => "dcmpl",
            OpCode::dcmpg => "dcmpg",
            OpCode::ifeq => "ifeq",
            OpCode::ifne => "ifne",
            OpCode::iflt => "iflt",
            OpCode::ifge => "ifge",
            OpCode::ifgt => "ifgt",
            OpCode::ifle => "ifle",
            OpCode::if_icmpeq => "if_icmpeq",
            OpCode::if_icmpne => "if_icmpne",
            OpCode::if_icmplt => "if_icmplt",
            OpCode::if_icmpge => "if_icmpge",
            OpCode::if_icmpgt => "if_icmpgt",
            OpCode::if_icmple => "if_icmple",
            OpCode::if_acmpeq => "if_acmpeq",
            OpCode::if_acmpne => "if_acmpne",
            OpCode::goto => "goto",
            OpCode::jsr => "jsr",
            OpCode::ret => "ret",
            OpCode::tableswitch => "tableswitch",
            OpCode::lookupswitch => "lookupswitch",
            OpCode::ireturn => "ireturn",
            OpCode::lreturn => "lreturn",
            OpCode::freturn => "freturn",
            OpCode::dreturn => "dreturn",
            OpCode::areturn => "areturn",
            OpCode::return_void => "return",
            OpCode::getstatic => "getstatic",
            OpCode::putstatic => "putstatic",
            OpCode::getfield => "getfield",
            OpCode::putfield => "putfield",
            OpCode::invokevirtual => "invokevirtual",
            OpCode::invokespecial => "invokespecial",
            OpCode::invokestatic => "invokestatic",
            OpCode::invokeinterface => "invokeinterface",
            OpCode::invokedynamic => "invokedynamic",
            OpCode::new => "new",
            OpCode::newarray => "newarray",
            OpCode::anewarray => "anewarray",
            OpCode::arraylength => "arraylength",
            OpCode::athrow => "athrow",
            OpCode::checkcast => "checkcast",
            OpCode::instanceof => "instanceof",
            OpCode::monitorenter => "monitorenter",
            OpCode::monitorexit => "monitorexit",
            OpCode::wide => "wide",
            OpCode::multianewarray => "multianewarray",
            OpCode::ifnull => "ifnull",
            OpCode::ifnonnull => "ifnonnull",
            OpCode::goto_w => "goto_w",
            OpCode::jsr_w => "jsr_w",
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode;

    #[test]
    fn t_opcode() {
        assert_eq!(OpCode::nop, OpCode::from(0));
        assert_eq!(OpCode::aconst_null, OpCode::from(1));
        assert_eq!(OpCode::iconst_m1, OpCode::from(2));
        assert_eq!(OpCode::iconst_0, OpCode::from(3));
        assert_eq!(OpCode::iconst_1, OpCode::from(4));
        assert_eq!(OpCode::iconst_2, OpCode::from(5));
        assert_eq!(OpCode::iconst_3, OpCode::from(6));
        assert_eq!(OpCode::iconst_4, OpCode::from(7));
        assert_eq!(OpCode::iconst_5, OpCode::from(8));
        assert_eq!(OpCode::lconst_0, OpCode::from(9));
        assert_eq!(OpCode::lconst_1, OpCode::from(10));
        assert_eq!(OpCode::fconst_0, OpCode::from(11));
        assert_eq!(OpCode::fconst_1, OpCode::from(12));
        assert_eq!(OpCode::fconst_2, OpCode::from(13));
        assert_eq!(OpCode::dconst_0, OpCode::from(14));
        assert_eq!(OpCode::dconst_1, OpCode::from(15));
        assert_eq!(OpCode::bipush, OpCode::from(16));
        assert_eq!(OpCode::sipush, OpCode::from(17));
        assert_eq!(OpCode::ldc, OpCode::from(18));
        assert_eq!(OpCode::ldc_w, OpCode::from(19));
        assert_eq!(OpCode::ldc2_w, OpCode::from(20));
        assert_eq!(OpCode::iload, OpCode::from(21));
        assert_eq!(OpCode::lload, OpCode::from(22));
        assert_eq!(OpCode::fload, OpCode::from(23));
        assert_eq!(OpCode::dload, OpCode::from(24));
        assert_eq!(OpCode::aload, OpCode::from(25));
        assert_eq!(OpCode::iload_0, OpCode::from(26));
        assert_eq!(OpCode::iload_1, OpCode::from(27));
        assert_eq!(OpCode::iload_2, OpCode::from(28));
        assert_eq!(OpCode::iload_3, OpCode::from(29));
        assert_eq!(OpCode::lload_0, OpCode::from(30));
        assert_eq!(OpCode::lload_1, OpCode::from(31));
        assert_eq!(OpCode::lload_2, OpCode::from(32));
        assert_eq!(OpCode::lload_3, OpCode::from(33));
        assert_eq!(OpCode::fload_0, OpCode::from(34));
        assert_eq!(OpCode::fload_1, OpCode::from(35));
        assert_eq!(OpCode::fload_2, OpCode::from(36));
        assert_eq!(OpCode::fload_3, OpCode::from(37));
        assert_eq!(OpCode::dload_0, OpCode::from(38));
        assert_eq!(OpCode::dload_1, OpCode::from(39));
        assert_eq!(OpCode::dload_2, OpCode::from(40));
        assert_eq!(OpCode::dload_3, OpCode::from(41));
        assert_eq!(OpCode::aload_0, OpCode::from(42));
        assert_eq!(OpCode::aload_1, OpCode::from(43));
        assert_eq!(OpCode::aload_2, OpCode::from(44));
        assert_eq!(OpCode::aload_3, OpCode::from(45));
        assert_eq!(OpCode::iaload, OpCode::from(46));
        assert_eq!(OpCode::laload, OpCode::from(47));
        assert_eq!(OpCode::faload, OpCode::from(48));
        assert_eq!(OpCode::daload, OpCode::from(49));
        assert_eq!(OpCode::aaload, OpCode::from(50));
        assert_eq!(OpCode::baload, OpCode::from(51));
        assert_eq!(OpCode::caload, OpCode::from(52));
        assert_eq!(OpCode::saload, OpCode::from(53));
        assert_eq!(OpCode::istore, OpCode::from(54));
        assert_eq!(OpCode::lstore, OpCode::from(55));
        assert_eq!(OpCode::fstore, OpCode::from(56));
        assert_eq!(OpCode::dstore, OpCode::from(57));
        assert_eq!(OpCode::astore, OpCode::from(58));
        assert_eq!(OpCode::istore_0, OpCode::from(59));
        assert_eq!(OpCode::istore_1, OpCode::from(60));
        assert_eq!(OpCode::istore_2, OpCode::from(61));
        assert_eq!(OpCode::istore_3, OpCode::from(62));
        assert_eq!(OpCode::lstore_0, OpCode::from(63));
        assert_eq!(OpCode::lstore_1, OpCode::from(64));
        assert_eq!(OpCode::lstore_2, OpCode::from(65));
        assert_eq!(OpCode::lstore_3, OpCode::from(66));
        assert_eq!(OpCode::fstore_0, OpCode::from(67));
        assert_eq!(OpCode::fstore_1, OpCode::from(68));
        assert_eq!(OpCode::fstore_2, OpCode::from(69));
        assert_eq!(OpCode::fstore_3, OpCode::from(70));
        assert_eq!(OpCode::dstore_0, OpCode::from(71));
        assert_eq!(OpCode::dstore_1, OpCode::from(72));
        assert_eq!(OpCode::dstore_2, OpCode::from(73));
        assert_eq!(OpCode::dstore_3, OpCode::from(74));
        assert_eq!(OpCode::astore_0, OpCode::from(75));
        assert_eq!(OpCode::astore_1, OpCode::from(76));
        assert_eq!(OpCode::astore_2, OpCode::from(77));
        assert_eq!(OpCode::astore_3, OpCode::from(78));
        assert_eq!(OpCode::iastore, OpCode::from(79));
        assert_eq!(OpCode::lastore, OpCode::from(80));
        assert_eq!(OpCode::fastore, OpCode::from(81));
        assert_eq!(OpCode::dastore, OpCode::from(82));
        assert_eq!(OpCode::aastore, OpCode::from(83));
        assert_eq!(OpCode::bastore, OpCode::from(84));
        assert_eq!(OpCode::castore, OpCode::from(85));
        assert_eq!(OpCode::sastore, OpCode::from(86));
        assert_eq!(OpCode::pop, OpCode::from(87));
        assert_eq!(OpCode::pop2, OpCode::from(88));
        assert_eq!(OpCode::dup, OpCode::from(89));
        assert_eq!(OpCode::dup_x1, OpCode::from(90));
        assert_eq!(OpCode::dup_x2, OpCode::from(91));
        assert_eq!(OpCode::dup2, OpCode::from(92));
        assert_eq!(OpCode::dup2_x1, OpCode::from(93));
        assert_eq!(OpCode::dup2_x2, OpCode::from(94));
        assert_eq!(OpCode::swap, OpCode::from(95));
        assert_eq!(OpCode::iadd, OpCode::from(96));
        assert_eq!(OpCode::ladd, OpCode::from(97));
        assert_eq!(OpCode::fadd, OpCode::from(98));
        assert_eq!(OpCode::dadd, OpCode::from(99));
        assert_eq!(OpCode::isub, OpCode::from(100));
        assert_eq!(OpCode::lsub, OpCode::from(101));
        assert_eq!(OpCode::fsub, OpCode::from(102));
        assert_eq!(OpCode::dsub, OpCode::from(103));
        assert_eq!(OpCode::imul, OpCode::from(104));
        assert_eq!(OpCode::lmul, OpCode::from(105));
        assert_eq!(OpCode::fmul, OpCode::from(106));
        assert_eq!(OpCode::dmul, OpCode::from(107));
        assert_eq!(OpCode::idiv, OpCode::from(108));
        assert_eq!(OpCode::ldiv, OpCode::from(109));
        assert_eq!(OpCode::fdiv, OpCode::from(110));
        assert_eq!(OpCode::ddiv, OpCode::from(111));
        assert_eq!(OpCode::irem, OpCode::from(112));
        assert_eq!(OpCode::lrem, OpCode::from(113));
        assert_eq!(OpCode::frem, OpCode::from(114));
        assert_eq!(OpCode::drem, OpCode::from(115));
        assert_eq!(OpCode::ineg, OpCode::from(116));
        assert_eq!(OpCode::lneg, OpCode::from(117));
        assert_eq!(OpCode::fneg, OpCode::from(118));
        assert_eq!(OpCode::dneg, OpCode::from(119));
        assert_eq!(OpCode::ishl, OpCode::from(120));
        assert_eq!(OpCode::lshl, OpCode::from(121));
        assert_eq!(OpCode::ishr, OpCode::from(122));
        assert_eq!(OpCode::lshr, OpCode::from(123));
        assert_eq!(OpCode::iushr, OpCode::from(124));
        assert_eq!(OpCode::lushr, OpCode::from(125));
        assert_eq!(OpCode::iand, OpCode::from(126));
        assert_eq!(OpCode::land, OpCode::from(127));
        assert_eq!(OpCode::ior, OpCode::from(128));
        assert_eq!(OpCode::lor, OpCode::from(129));
        assert_eq!(OpCode::ixor, OpCode::from(130));
        assert_eq!(OpCode::lxor, OpCode::from(131));
        assert_eq!(OpCode::iinc, OpCode::from(132));
        assert_eq!(OpCode::i2l, OpCode::from(133));
        assert_eq!(OpCode::i2f, OpCode::from(134));
        assert_eq!(OpCode::i2d, OpCode::from(135));
        assert_eq!(OpCode::l2i, OpCode::from(136));
        assert_eq!(OpCode::l2f, OpCode::from(137));
        assert_eq!(OpCode::l2d, OpCode::from(138));
        assert_eq!(OpCode::f2i, OpCode::from(139));
        assert_eq!(OpCode::f2l, OpCode::from(140));
        assert_eq!(OpCode::f2d, OpCode::from(141));
        assert_eq!(OpCode::d2i, OpCode::from(142));
        assert_eq!(OpCode::d2l, OpCode::from(143));
        assert_eq!(OpCode::d2f, OpCode::from(144));
        assert_eq!(OpCode::i2b, OpCode::from(145));
        assert_eq!(OpCode::i2c, OpCode::from(146));
        assert_eq!(OpCode::i2s, OpCode::from(147));
        assert_eq!(OpCode::lcmp, OpCode::from(148));
        assert_eq!(OpCode::fcmpl, OpCode::from(149));
        assert_eq!(OpCode::fcmpg, OpCode::from(150));
        assert_eq!(OpCode::dcmpl, OpCode::from(151));
        assert_eq!(OpCode::dcmpg, OpCode::from(152));
        assert_eq!(OpCode::ifeq, OpCode::from(153));
        assert_eq!(OpCode::ifne, OpCode::from(154));
        assert_eq!(OpCode::iflt, OpCode::from(155));
        assert_eq!(OpCode::ifge, OpCode::from(156));
        assert_eq!(OpCode::ifgt, OpCode::from(157));
        assert_eq!(OpCode::ifle, OpCode::from(158));
        assert_eq!(OpCode::if_icmpeq, OpCode::from(159));
        assert_eq!(OpCode::if_icmpne, OpCode::from(160));
        assert_eq!(OpCode::if_icmplt, OpCode::from(161));
        assert_eq!(OpCode::if_icmpge, OpCode::from(162));
        assert_eq!(OpCode::if_icmpgt, OpCode::from(163));
        assert_eq!(OpCode::if_icmple, OpCode::from(164));
        assert_eq!(OpCode::if_acmpeq, OpCode::from(165));
        assert_eq!(OpCode::if_acmpne, OpCode::from(166));
        assert_eq!(OpCode::goto, OpCode::from(167));
        assert_eq!(OpCode::jsr, OpCode::from(168));
        assert_eq!(OpCode::ret, OpCode::from(169));
        assert_eq!(OpCode::tableswitch, OpCode::from(170));
        assert_eq!(OpCode::lookupswitch, OpCode::from(171));
        assert_eq!(OpCode::ireturn, OpCode::from(172));
        assert_eq!(OpCode::lreturn, OpCode::from(173));
        assert_eq!(OpCode::freturn, OpCode::from(174));
        assert_eq!(OpCode::dreturn, OpCode::from(175));
        assert_eq!(OpCode::areturn, OpCode::from(176));
        assert_eq!(OpCode::return_void, OpCode::from(177));
        assert_eq!(OpCode::getstatic, OpCode::from(178));
        assert_eq!(OpCode::putstatic, OpCode::from(179));
        assert_eq!(OpCode::getfield, OpCode::from(180));
        assert_eq!(OpCode::putfield, OpCode::from(181));
        assert_eq!(OpCode::invokevirtual, OpCode::from(182));
        assert_eq!(OpCode::invokespecial, OpCode::from(183));
        assert_eq!(OpCode::invokestatic, OpCode::from(184));
        assert_eq!(OpCode::invokeinterface, OpCode::from(185));
        assert_eq!(OpCode::invokedynamic, OpCode::from(186));
        assert_eq!(OpCode::new, OpCode::from(187));
        assert_eq!(OpCode::newarray, OpCode::from(188));
        assert_eq!(OpCode::anewarray, OpCode::from(189));
        assert_eq!(OpCode::arraylength, OpCode::from(190));
        assert_eq!(OpCode::athrow, OpCode::from(191));
        assert_eq!(OpCode::checkcast, OpCode::from(192));
        assert_eq!(OpCode::instanceof, OpCode::from(193));
        assert_eq!(OpCode::monitorenter, OpCode::from(194));
        assert_eq!(OpCode::monitorexit, OpCode::from(195));
        assert_eq!(OpCode::wide, OpCode::from(196));
        assert_eq!(OpCode::multianewarray, OpCode::from(197));
        assert_eq!(OpCode::ifnull, OpCode::from(198));
        assert_eq!(OpCode::ifnonnull, OpCode::from(199));
        assert_eq!(OpCode::goto_w, OpCode::from(200));
        assert_eq!(OpCode::jsr_w, OpCode::from(201));
        assert_eq!(OpCode::breakpoint, OpCode::from(202));
        //        assert_eq!(OpCode::, OpCode::from(203));
        //        assert_eq!(OpCode::, OpCode::from(204));
        //        assert_eq!(OpCode::, OpCode::from(205));
        //        assert_eq!(OpCode::, OpCode::from(206));
        //        assert_eq!(OpCode::, OpCode::from(207));
        //        assert_eq!(OpCode::, OpCode::from(208));
        //        assert_eq!(OpCode::, OpCode::from(209));
        //        assert_eq!(OpCode::, OpCode::from(210));
        //        assert_eq!(OpCode::, OpCode::from(211));
        //        assert_eq!(OpCode::, OpCode::from(212));
        //        assert_eq!(OpCode::, OpCode::from(213));
        //        assert_eq!(OpCode::, OpCode::from(214));
        //        assert_eq!(OpCode::, OpCode::from(215));
        //        assert_eq!(OpCode::, OpCode::from(216));
        //        assert_eq!(OpCode::, OpCode::from(217));
        //        assert_eq!(OpCode::, OpCode::from(218));
        //        assert_eq!(OpCode::, OpCode::from(219));
        //        assert_eq!(OpCode::, OpCode::from(220));
        //        assert_eq!(OpCode::, OpCode::from(221));
        //        assert_eq!(OpCode::, OpCode::from(222));
        //        assert_eq!(OpCode::, OpCode::from(223));
        //        assert_eq!(OpCode::, OpCode::from(224));
        //        assert_eq!(OpCode::, OpCode::from(225));
        //        assert_eq!(OpCode::, OpCode::from(226));
        //        assert_eq!(OpCode::, OpCode::from(227));
        //        assert_eq!(OpCode::, OpCode::from(228));
        //        assert_eq!(OpCode::, OpCode::from(229));
        //        assert_eq!(OpCode::, OpCode::from(230));
        //        assert_eq!(OpCode::, OpCode::from(231));
        //        assert_eq!(OpCode::, OpCode::from(232));
        //        assert_eq!(OpCode::, OpCode::from(233));
        //        assert_eq!(OpCode::, OpCode::from(234));
        //        assert_eq!(OpCode::, OpCode::from(235));
        //        assert_eq!(OpCode::, OpCode::from(236));
        //        assert_eq!(OpCode::, OpCode::from(237));
        //        assert_eq!(OpCode::, OpCode::from(238));
        //        assert_eq!(OpCode::, OpCode::from(239));
        //        assert_eq!(OpCode::, OpCode::from(240));
        //        assert_eq!(OpCode::, OpCode::from(241));
        //        assert_eq!(OpCode::, OpCode::from(242));
        //        assert_eq!(OpCode::, OpCode::from(243));
        //        assert_eq!(OpCode::, OpCode::from(244));
        //        assert_eq!(OpCode::, OpCode::from(245));
        //        assert_eq!(OpCode::, OpCode::from(246));
        //        assert_eq!(OpCode::, OpCode::from(247));
        //        assert_eq!(OpCode::, OpCode::from(248));
        //        assert_eq!(OpCode::, OpCode::from(249));
        //        assert_eq!(OpCode::, OpCode::from(250));
        //        assert_eq!(OpCode::, OpCode::from(251));
        //        assert_eq!(OpCode::, OpCode::from(252));
        //        assert_eq!(OpCode::, OpCode::from(253));
        assert_eq!(OpCode::impdep1, OpCode::from(254));
        assert_eq!(OpCode::impdep2, OpCode::from(255));
        //        assert_eq!(OpCode::, OpCode::from(256));
    }
}
