#[derive(Serialize)]
pub struct CodeSerde {
    pub stack: usize,
    pub locals: usize,
    pub args_size: usize,
    pub code: Vec<ByteCodeSerde>,
}

#[derive(Serialize)]
pub struct ByteCodeSerde {
    pub pc: usize,
    pub desc: &'static str,
    pub cp_index: Option<usize>,
    pub comment: String
}