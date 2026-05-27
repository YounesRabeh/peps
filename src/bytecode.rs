#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    LoadConst(Value),
    LoadVar(String),
    StoreVar(String),

    Add,
    Sub,
    Mul,
    Div,

    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    MakeList(usize),
    Print,

    Jump(usize),
    JumpIfFalse(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(i64),
    Str(String),
    Bool(bool),
    Emoji(String),
}
