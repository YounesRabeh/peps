//! Peps bytecode instructions and constant values.

use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    LoadConst(Value),
    LoadVar(String),
    StoreVar(String),
    LoadLocal(String),
    StoreLocal(String),

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
    Pop,

    Call { target: usize, arity: usize },
    Return,

    Jump(usize),
    JumpIfFalse(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(BigInt),
    Float(f64),
    Str(String),
    Bool(bool),
    Emoji(String),
}
