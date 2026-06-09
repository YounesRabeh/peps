//! Peps bytecode instructions and constant values.

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
    ListLen,
    ListGet,
    ListAppend,
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
