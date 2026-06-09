//! Stack-based bytecode runner for compiled Peps programs.

use std::collections::HashMap;

use crate::{
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
};

pub const DEFAULT_STEP_LIMIT: usize = 100_000;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Num(i64),
    Str(String),
    Bool(bool),
    Emoji(String),
    List(Vec<RuntimeValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
    pub output: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn execute(instructions: &[Instruction]) -> Result<Vec<String>, RunError> {
    execute_with_step_limit(instructions, DEFAULT_STEP_LIMIT)
}

pub fn execute_with_step_limit(
    instructions: &[Instruction],
    step_limit: usize,
) -> Result<Vec<String>, RunError> {
    let mut vm = Vm {
        instructions,
        ip: 0,
        stack: Vec::new(),
        variables: HashMap::new(),
        output: Vec::new(),
        steps: 0,
        step_limit,
    };
    vm.run()
}

struct Vm<'a> {
    instructions: &'a [Instruction],
    ip: usize,
    stack: Vec<RuntimeValue>,
    variables: HashMap<String, RuntimeValue>,
    output: Vec<String>,
    steps: usize,
    step_limit: usize,
}

impl Vm<'_> {
    fn run(&mut self) -> Result<Vec<String>, RunError> {
        while self.ip < self.instructions.len() {
            if self.steps >= self.step_limit {
                return self.fail(
                    "execution step limit reached; the program may contain a non-terminating loop",
                );
            }
            self.steps += 1;

            match self.instructions[self.ip].clone() {
                Instruction::LoadConst(value) => {
                    self.stack.push(RuntimeValue::from(value));
                    self.ip += 1;
                }
                Instruction::LoadVar(name) => {
                    let Some(value) = self.variables.get(&name).cloned() else {
                        return self.fail(format!("runtime variable {} is not declared", name));
                    };
                    self.stack.push(value);
                    self.ip += 1;
                }
                Instruction::StoreVar(name) => {
                    let value = self.pop("store variable")?;
                    self.variables.insert(name, value);
                    self.ip += 1;
                }
                Instruction::Add => self.add_values()?,
                Instruction::Sub => self.binary_num("subtract", |left, right| left - right)?,
                Instruction::Mul => self.binary_num("multiply", |left, right| left * right)?,
                Instruction::Div => {
                    let right = self.pop_num("divide")?;
                    let left = self.pop_num("divide")?;
                    if right == 0 {
                        return self.fail("division by zero");
                    }
                    self.stack.push(RuntimeValue::Num(left / right));
                    self.ip += 1;
                }
                Instruction::Eq => self.equality(false)?,
                Instruction::NotEq => self.equality(true)?,
                Instruction::Lt => self.compare_num("compare", |left, right| left < right)?,
                Instruction::Gt => self.compare_num("compare", |left, right| left > right)?,
                Instruction::LtEq => self.compare_num("compare", |left, right| left <= right)?,
                Instruction::GtEq => self.compare_num("compare", |left, right| left >= right)?,
                Instruction::MakeList(count) => {
                    if self.stack.len() < count {
                        return self.fail("not enough values on the stack to build list");
                    }
                    let start = self.stack.len() - count;
                    let elements = self.stack.split_off(start);
                    self.stack.push(RuntimeValue::List(elements));
                    self.ip += 1;
                }
                Instruction::ListLen => {
                    let value = self.pop("list length")?;
                    let RuntimeValue::List(elements) = value else {
                        return self.fail("list length requires a list value");
                    };
                    self.stack.push(RuntimeValue::Num(elements.len() as i64));
                    self.ip += 1;
                }
                Instruction::ListGet => {
                    let index = self.pop_num("list index")?;
                    let list = self.pop("list index")?;
                    let RuntimeValue::List(elements) = list else {
                        return self.fail("list index requires a list value");
                    };
                    if index < 0 || index as usize >= elements.len() {
                        return self.fail(format!("list index {} is out of bounds", index));
                    }
                    self.stack.push(elements[index as usize].clone());
                    self.ip += 1;
                }
                Instruction::ListAppend => {
                    let value = self.pop("list append")?;
                    let list = self.pop("list append")?;
                    let RuntimeValue::List(mut elements) = list else {
                        return self.fail("list append requires a list value");
                    };
                    match value {
                        RuntimeValue::List(values) => elements.extend(values),
                        value => elements.push(value),
                    }
                    self.stack.push(RuntimeValue::List(elements));
                    self.ip += 1;
                }
                Instruction::Print => {
                    let value = self.pop("print")?;
                    self.output.push(format_runtime_value(&value));
                    self.ip += 1;
                }
                Instruction::Jump(target) => {
                    self.validate_jump(target)?;
                    self.ip = target;
                }
                Instruction::JumpIfFalse(target) => {
                    self.validate_jump(target)?;
                    let condition = self.pop_bool("conditional jump")?;
                    if condition {
                        self.ip += 1;
                    } else {
                        self.ip = target;
                    }
                }
            }
        }

        Ok(std::mem::take(&mut self.output))
    }

    fn binary_num(
        &mut self,
        operation: &'static str,
        apply: impl FnOnce(i64, i64) -> i64,
    ) -> Result<(), RunError> {
        let right = self.pop_num(operation)?;
        let left = self.pop_num(operation)?;
        self.stack.push(RuntimeValue::Num(apply(left, right)));
        self.ip += 1;
        Ok(())
    }

    fn add_values(&mut self) -> Result<(), RunError> {
        let right = self.pop("add")?;
        let left = self.pop("add")?;
        match (left, right) {
            (RuntimeValue::Num(left), RuntimeValue::Num(right)) => {
                self.stack.push(RuntimeValue::Num(left + right));
            }
            (RuntimeValue::Str(left), RuntimeValue::Str(right)) => {
                self.stack.push(RuntimeValue::Str(format!("{}{}", left, right)));
            }
            _ => return self.fail("add requires matching num or text values"),
        }
        self.ip += 1;
        Ok(())
    }

    fn compare_num(
        &mut self,
        operation: &'static str,
        apply: impl FnOnce(i64, i64) -> bool,
    ) -> Result<(), RunError> {
        let right = self.pop_num(operation)?;
        let left = self.pop_num(operation)?;
        self.stack.push(RuntimeValue::Bool(apply(left, right)));
        self.ip += 1;
        Ok(())
    }

    fn equality(&mut self, invert: bool) -> Result<(), RunError> {
        let right = self.pop("compare equality")?;
        let left = self.pop("compare equality")?;
        let equal = match (left, right) {
            (RuntimeValue::Num(left), RuntimeValue::Num(right)) => left == right,
            (RuntimeValue::Str(left), RuntimeValue::Str(right)) => left == right,
            (RuntimeValue::Bool(left), RuntimeValue::Bool(right)) => left == right,
            (RuntimeValue::Emoji(left), RuntimeValue::Emoji(right)) => left == right,
            _ => return self.fail("runtime equality requires matching scalar values"),
        };
        self.stack.push(RuntimeValue::Bool(if invert {
            !equal
        } else {
            equal
        }));
        self.ip += 1;
        Ok(())
    }

    fn pop(&mut self, operation: &'static str) -> Result<RuntimeValue, RunError> {
        self.stack
            .pop()
            .ok_or_else(|| self.error(format!("stack underflow during {}", operation)))
    }

    fn pop_num(&mut self, operation: &'static str) -> Result<i64, RunError> {
        match self.pop(operation)? {
            RuntimeValue::Num(value) => Ok(value),
            _ => Err(self.error(format!("{} requires a num value", operation))),
        }
    }

    fn pop_bool(&mut self, operation: &'static str) -> Result<bool, RunError> {
        match self.pop(operation)? {
            RuntimeValue::Bool(value) => Ok(value),
            _ => Err(self.error(format!("{} requires a bool value", operation))),
        }
    }

    fn validate_jump(&self, target: usize) -> Result<(), RunError> {
        if target <= self.instructions.len() {
            Ok(())
        } else {
            Err(self.error(format!("invalid jump target {}", target)))
        }
    }

    fn fail<T>(&mut self, message: impl Into<String>) -> Result<T, RunError> {
        Err(self.error(message))
    }

    fn error(&self, message: impl Into<String>) -> RunError {
        RunError {
            output: self.output.clone(),
            diagnostics: vec![Diagnostic::new(message)],
        }
    }
}

impl From<Value> for RuntimeValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Num(value) => RuntimeValue::Num(value),
            Value::Str(value) => RuntimeValue::Str(value),
            Value::Bool(value) => RuntimeValue::Bool(value),
            Value::Emoji(value) => RuntimeValue::Emoji(value),
        }
    }
}

fn format_runtime_value(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::Num(value) => value.to_string(),
        RuntimeValue::Str(value) => value.clone(),
        RuntimeValue::Bool(true) => "✅".to_string(),
        RuntimeValue::Bool(false) => "❌".to_string(),
        RuntimeValue::Emoji(value) => value.clone(),
        RuntimeValue::List(elements) => {
            let items = elements
                .iter()
                .map(format_runtime_value)
                .collect::<Vec<_>>()
                .join(" ");
            format!("📚 {} 📚", items)
        }
    }
}
