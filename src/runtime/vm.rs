//! Stack-based bytecode runner for compiled Peps programs.

use std::collections::HashMap;

use crate::{
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
};

/// Default maximum number of bytecode instructions a program may execute.
///
/// The limit prevents non-terminating loops from running forever in the runtime.
pub const DEFAULT_STEP_LIMIT: usize = 100_000;

/// Runtime representation of values stored on the VM stack and in variables.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// Integer numeric value.
    Num(i64),
    /// Text value.
    Str(String),
    /// Boolean value.
    Bool(bool),
    /// Emoji literal value.
    Emoji(String),
    /// List value containing runtime values in source order.
    List(Vec<RuntimeValue>),
}

/// Runtime failure with any output produced before the error.
#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
    /// Lines printed before execution failed.
    pub output: Vec<String>,
    /// Runtime diagnostics explaining the failure.
    pub diagnostics: Vec<Diagnostic>,
}

/// Execute bytecode with [`DEFAULT_STEP_LIMIT`].
pub fn execute(instructions: &[Instruction]) -> Result<Vec<String>, RunError> {
    execute_with_step_limit(instructions, DEFAULT_STEP_LIMIT)
}

/// Execute bytecode with a caller-provided instruction step limit.
///
/// The returned vector contains each value printed by the program. If execution
/// fails, [`RunError::output`] preserves any prints that happened first.
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
    /// Compiled bytecode being executed.
    instructions: &'a [Instruction],
    /// Current instruction pointer.
    ip: usize,
    /// Operand stack used by bytecode instructions.
    stack: Vec<RuntimeValue>,
    /// Runtime variable storage keyed by Peps variable name.
    variables: HashMap<String, RuntimeValue>,
    /// Formatted print output accumulated during execution.
    output: Vec<String>,
    /// Number of instructions executed so far.
    steps: usize,
    /// Maximum number of instructions allowed for this run.
    step_limit: usize,
}

impl Vm<'_> {
    /// Run instructions until completion, an error, or the step limit.
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

    /// Apply a numeric binary operation to the top two stack values.
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

    /// Add numbers or concatenate text values.
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

    /// Apply a numeric comparison and push the resulting boolean.
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

    /// Compare scalar runtime values for equality or inequality.
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

    /// Pop one value from the operand stack.
    fn pop(&mut self, operation: &'static str) -> Result<RuntimeValue, RunError> {
        self.stack
            .pop()
            .ok_or_else(|| self.error(format!("stack underflow during {}", operation)))
    }

    /// Pop and type-check a numeric stack value.
    fn pop_num(&mut self, operation: &'static str) -> Result<i64, RunError> {
        match self.pop(operation)? {
            RuntimeValue::Num(value) => Ok(value),
            _ => Err(self.error(format!("{} requires a num value", operation))),
        }
    }

    /// Pop and type-check a boolean stack value.
    fn pop_bool(&mut self, operation: &'static str) -> Result<bool, RunError> {
        match self.pop(operation)? {
            RuntimeValue::Bool(value) => Ok(value),
            _ => Err(self.error(format!("{} requires a bool value", operation))),
        }
    }

    /// Ensure a jump target points inside the instruction stream or just past it.
    fn validate_jump(&self, target: usize) -> Result<(), RunError> {
        if target <= self.instructions.len() {
            Ok(())
        } else {
            Err(self.error(format!("invalid jump target {}", target)))
        }
    }

    /// Return a runtime error result from the current VM state.
    fn fail<T>(&mut self, message: impl Into<String>) -> Result<T, RunError> {
        Err(self.error(message))
    }

    /// Build a runtime error while preserving output produced so far.
    fn error(&self, message: impl Into<String>) -> RunError {
        RunError {
            output: self.output.clone(),
            diagnostics: vec![Diagnostic::new(message)],
        }
    }
}

impl From<Value> for RuntimeValue {
    /// Convert a bytecode constant into its runtime representation.
    fn from(value: Value) -> Self {
        match value {
            Value::Num(value) => RuntimeValue::Num(value),
            Value::Str(value) => RuntimeValue::Str(value),
            Value::Bool(value) => RuntimeValue::Bool(value),
            Value::Emoji(value) => RuntimeValue::Emoji(value),
        }
    }
}

/// Format a runtime value the way Peps `print` emits it.
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
