//! Stack-based bytecode runner for compiled Peps programs.

use std::collections::{HashMap, VecDeque};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    ast::{ConversionKind, InputKind},
    bytecode::{Instruction, Value},
    diagnostic::Diagnostic,
    source::Span,
};

/// Maximum instructions used for browser IDE executions.
pub const IDE_STEP_LIMIT: usize = 100_000;
/// Diagnostic prefix used when execution needs another input value.
pub const INPUT_REQUIRED_PREFIX: &str = "input required: ";

/// Optional instruction limit for a VM execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionLimit {
    /// Run until the program completes or encounters a runtime error.
    Unlimited,
    /// Stop after the given number of instructions.
    Steps(usize),
}

/// Runtime representation of values stored on the VM stack and in variables.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// Integer numeric value.
    Num(BigInt),
    /// Floating-point numeric value.
    Float(f64),
    /// Text value.
    Str(String),
    /// Boolean value.
    Bool(bool),
    /// Emoji literal value.
    Emoji(String),
    /// List value containing runtime values in source order.
    List(Vec<RuntimeValue>),
    /// Ordered text-keyed map.
    Map(Vec<(String, RuntimeValue)>),
}

/// Runtime failure with any output produced before the error.
#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
    /// Lines printed before execution failed.
    pub output: Vec<String>,
    /// Runtime diagnostics explaining the failure.
    pub diagnostics: Vec<Diagnostic>,
}

/// Execute bytecode without an instruction limit.
pub fn execute(instructions: &[Instruction]) -> Result<Vec<String>, RunError> {
    execute_with_inputs_and_limit(
        instructions,
        std::iter::empty::<String>(),
        ExecutionLimit::Unlimited,
    )
}

/// Execute bytecode with a caller-provided instruction step limit.
///
/// The returned vector contains each value printed by the program. If execution
/// fails, [`RunError::output`] preserves any prints that happened first.
pub fn execute_with_step_limit(
    instructions: &[Instruction],
    step_limit: usize,
) -> Result<Vec<String>, RunError> {
    execute_with_inputs_and_limit(
        instructions,
        std::iter::empty::<String>(),
        ExecutionLimit::Steps(step_limit),
    )
}

/// Execute bytecode with an explicit instruction-limit policy.
pub fn execute_with_limit(
    instructions: &[Instruction],
    execution_limit: ExecutionLimit,
) -> Result<Vec<String>, RunError> {
    execute_with_inputs_and_limit(instructions, std::iter::empty::<String>(), execution_limit)
}

/// Execute bytecode using input lines supplied in source order.
pub fn execute_with_inputs<I, S>(
    instructions: &[Instruction],
    inputs: I,
) -> Result<Vec<String>, RunError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    execute_with_inputs_and_limit(instructions, inputs, ExecutionLimit::Unlimited)
}

/// Execute bytecode with queued inputs and an explicit instruction limit.
pub fn execute_with_inputs_and_limit<I, S>(
    instructions: &[Instruction],
    inputs: I,
    execution_limit: ExecutionLimit,
) -> Result<Vec<String>, RunError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut inputs = inputs.into_iter().map(Into::into).collect::<VecDeque<_>>();
    execute_with_input_reader_and_spans(instructions, None, execution_limit, |kind| {
        inputs
            .pop_front()
            .ok_or_else(|| format!("{}{}", INPUT_REQUIRED_PREFIX, kind.name()))
    })
}

/// Execute browser bytecode with source locations for runtime diagnostics.
pub(crate) fn execute_with_inputs_and_source_spans<I, S>(
    instructions: &[Instruction],
    source_spans: &[Option<Span>],
    inputs: I,
    execution_limit: ExecutionLimit,
) -> Result<Vec<String>, RunError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut inputs = inputs.into_iter().map(Into::into).collect::<VecDeque<_>>();
    execute_with_input_reader_and_spans(instructions, Some(source_spans), execution_limit, |kind| {
        inputs
            .pop_front()
            .ok_or_else(|| format!("{}{}", INPUT_REQUIRED_PREFIX, kind.name()))
    })
}

/// Execute bytecode with an on-demand input reader.
pub fn execute_with_input_reader<F>(
    instructions: &[Instruction],
    execution_limit: ExecutionLimit,
    read_input: F,
) -> Result<Vec<String>, RunError>
where
    F: FnMut(InputKind) -> Result<String, String>,
{
    execute_with_input_reader_and_spans(instructions, None, execution_limit, read_input)
}

fn execute_with_input_reader_and_spans<F>(
    instructions: &[Instruction],
    source_spans: Option<&[Option<Span>]>,
    execution_limit: ExecutionLimit,
    mut read_input: F,
) -> Result<Vec<String>, RunError>
where
    F: FnMut(InputKind) -> Result<String, String>,
{
    let mut vm = Vm {
        instructions,
        source_spans,
        ip: 0,
        stack: Vec::new(),
        globals: HashMap::new(),
        frames: vec![CallFrame {
            return_address: None,
            stack_base: 0,
            locals: HashMap::new(),
        }],
        output: Vec::new(),
        steps: 0,
        execution_limit,
    };
    vm.run(&mut read_input)
}

struct Vm<'a> {
    /// Compiled bytecode being executed.
    instructions: &'a [Instruction],
    /// Optional source location corresponding to each bytecode instruction.
    source_spans: Option<&'a [Option<Span>]>,
    /// Current instruction pointer.
    ip: usize,
    /// Operand stack used by bytecode instructions.
    stack: Vec<RuntimeValue>,
    /// Runtime variable storage keyed by Peps variable name.
    /// Top-level variables shared by main and every function invocation.
    globals: HashMap<String, RuntimeValue>,
    /// The main frame followed by one independent frame per active call.
    frames: Vec<CallFrame>,
    /// Formatted print output accumulated during execution.
    output: Vec<String>,
    /// Number of instructions executed so far.
    steps: usize,
    /// Whether this run has an instruction limit.
    execution_limit: ExecutionLimit,
}

struct CallFrame {
    return_address: Option<usize>,
    stack_base: usize,
    locals: HashMap<String, RuntimeValue>,
}

enum NumberValue {
    Integer(BigInt),
    Float(f64),
}

impl Vm<'_> {
    /// Run instructions until completion, an error, or the step limit.
    fn run<F>(&mut self, read_input: &mut F) -> Result<Vec<String>, RunError>
    where
        F: FnMut(InputKind) -> Result<String, String>,
    {
        while self.ip < self.instructions.len() {
            if let ExecutionLimit::Steps(step_limit) = self.execution_limit {
                if self.steps >= step_limit {
                    return self.fail(
                        "execution step limit reached; the program may contain a non-terminating loop",
                    );
                }
                self.steps += 1;
            }

            match self.instructions[self.ip].clone() {
                Instruction::LoadConst(value) => {
                    self.stack.push(RuntimeValue::from(value));
                    self.ip += 1;
                }
                Instruction::Input(kind) => {
                    let raw = read_input(kind).map_err(|message| self.error(message))?;
                    let value = self.parse_input(kind, raw)?;
                    self.stack.push(value);
                    self.ip += 1;
                }
                Instruction::Convert(kind) => self.convert_value(kind)?,
                Instruction::LoadVar(name) => {
                    let Some(value) = self.globals.get(&name).cloned() else {
                        return self.fail(format!("runtime variable {} is not declared", name));
                    };
                    self.stack.push(value);
                    self.ip += 1;
                }
                Instruction::StoreVar(name) => {
                    let value = self.pop("store variable")?;
                    self.globals.insert(name, value);
                    self.ip += 1;
                }
                Instruction::LoadLocal(name) => {
                    let Some(value) = self
                        .frames
                        .last()
                        .and_then(|frame| frame.locals.get(&name))
                        .cloned()
                    else {
                        return self.fail(format!("runtime local {} is not declared", name));
                    };
                    self.stack.push(value);
                    self.ip += 1;
                }
                Instruction::StoreLocal(name) => {
                    let value = self.pop("store local")?;
                    self.frames
                        .last_mut()
                        .expect("VM always has a root frame")
                        .locals
                        .insert(name, value);
                    self.ip += 1;
                }
                Instruction::Add => self.add_values()?,
                Instruction::Sub => self.binary_number(
                    "subtract",
                    |left, right| left - right,
                    |left, right| left - right,
                )?,
                Instruction::Mul => self.binary_number(
                    "multiply",
                    |left, right| left * right,
                    |left, right| left * right,
                )?,
                Instruction::Div => self.divide_numbers()?,
                Instruction::Eq => self.equality(false)?,
                Instruction::NotEq => self.equality(true)?,
                Instruction::Lt => self.compare_num(
                    "compare",
                    |left, right| left < right,
                    |left, right| left < right,
                )?,
                Instruction::Gt => self.compare_num(
                    "compare",
                    |left, right| left > right,
                    |left, right| left > right,
                )?,
                Instruction::LtEq => self.compare_num(
                    "compare",
                    |left, right| left <= right,
                    |left, right| left <= right,
                )?,
                Instruction::GtEq => self.compare_num(
                    "compare",
                    |left, right| left >= right,
                    |left, right| left >= right,
                )?,
                Instruction::MakeList(count) => {
                    if self.stack.len() < count {
                        return self.fail("not enough values on the stack to build list");
                    }
                    let start = self.stack.len() - count;
                    let elements = self.stack.split_off(start);
                    self.stack.push(RuntimeValue::List(elements));
                    self.ip += 1;
                }
                Instruction::MakeMap(count) => {
                    let value_count = count
                        .checked_mul(2)
                        .ok_or_else(|| self.error("map is too large"))?;
                    if self.stack.len() < value_count {
                        return self.fail("not enough values on the stack to build map");
                    }
                    let start = self.stack.len() - value_count;
                    let values = self.stack.split_off(start);
                    let mut entries = Vec::with_capacity(count);
                    for pair in values.chunks_exact(2) {
                        let RuntimeValue::Str(key) = &pair[0] else {
                            return self.fail("map keys must be text");
                        };
                        if let Some((_, existing)) =
                            entries.iter_mut().find(|(existing, _)| existing == key)
                        {
                            *existing = pair[1].clone();
                        } else {
                            entries.push((key.clone(), pair[1].clone()));
                        }
                    }
                    if let Some((_, expected)) = entries.first() {
                        if entries
                            .iter()
                            .any(|(_, value)| !same_runtime_type(expected, value))
                        {
                            return self.fail("map values must all have the same type");
                        }
                    }
                    self.stack.push(RuntimeValue::Map(entries));
                    self.ip += 1;
                }
                Instruction::ListLen => {
                    let value = self.pop("collection length")?;
                    let length = match value {
                        RuntimeValue::List(elements) => elements.len(),
                        RuntimeValue::Map(entries) => entries.len(),
                        RuntimeValue::Str(text) => text.graphemes(true).count(),
                        _ => return self.fail("length requires text, a list, or a map value"),
                    };
                    self.stack.push(RuntimeValue::Num(BigInt::from(length)));
                    self.ip += 1;
                }
                Instruction::ListGet => {
                    let key = self.pop("collection lookup")?;
                    let collection = self.pop("collection lookup")?;
                    let value = match (collection, key) {
                        (RuntimeValue::List(elements), RuntimeValue::Num(index)) => {
                            let Some(index_value) = index.to_usize() else {
                                return self.fail(format!("list index {} is out of bounds", index));
                            };
                            let Some(value) = elements.get(index_value).cloned() else {
                                return self.fail(format!("list index {} is out of bounds", index));
                            };
                            value
                        }
                        (RuntimeValue::List(_), _) => {
                            return self.fail("list index requires an integer value");
                        }
                        (RuntimeValue::Map(entries), RuntimeValue::Str(key)) => {
                            let Some((_, value)) = entries.iter().find(|(entry, _)| entry == &key)
                            else {
                                return self.fail(format!("map key {:?} was not found", key));
                            };
                            value.clone()
                        }
                        (RuntimeValue::Map(_), _) => {
                            return self.fail("map lookup requires a text key");
                        }
                        (RuntimeValue::Str(text), RuntimeValue::Num(index)) => {
                            let Some(index_value) = index.to_usize() else {
                                return self.fail(format!("text index {} is out of bounds", index));
                            };
                            let Some(value) = text.graphemes(true).nth(index_value) else {
                                return self.fail(format!("text index {} is out of bounds", index));
                            };
                            RuntimeValue::Str(value.to_string())
                        }
                        (RuntimeValue::Str(_), _) => {
                            return self.fail("text index requires an integer value");
                        }
                        _ => return self.fail("lookup requires text, a list, or a map value"),
                    };
                    self.stack.push(value);
                    self.ip += 1;
                }
                Instruction::MapHas => {
                    let key = self.pop("map key existence")?;
                    let map = self.pop("map key existence")?;
                    let exists = match (map, key) {
                        (RuntimeValue::Map(entries), RuntimeValue::Str(key)) => {
                            entries.iter().any(|(entry, _)| entry == &key)
                        }
                        (RuntimeValue::Map(_), _) => {
                            return self.fail("map key existence requires a text key");
                        }
                        _ => return self.fail("map key existence requires a map value"),
                    };
                    self.stack.push(RuntimeValue::Bool(exists));
                    self.ip += 1;
                }
                Instruction::ListAppend => {
                    let right = self.pop("collection append")?;
                    let left = self.pop("collection append")?;
                    let result = match (left, right) {
                        (RuntimeValue::List(mut elements), value) => {
                            let appended = match value {
                                RuntimeValue::List(values) => values,
                                value => vec![value],
                            };
                            if let Some(expected) = elements.first().or_else(|| appended.first()) {
                                if appended
                                    .iter()
                                    .any(|value| !same_runtime_type(expected, value))
                                {
                                    return self.fail(
                                        "list append requires values matching the list element type",
                                    );
                                }
                            }
                            elements.extend(appended);
                            RuntimeValue::List(elements)
                        }
                        (RuntimeValue::Map(mut entries), RuntimeValue::Map(appended)) => {
                            if let Some(expected) = entries
                                .first()
                                .map(|(_, value)| value)
                                .or_else(|| appended.first().map(|(_, value)| value))
                            {
                                if appended
                                    .iter()
                                    .any(|(_, value)| !same_runtime_type(expected, value))
                                {
                                    return self.fail(
                                        "map merge requires values matching the map value type",
                                    );
                                }
                            }
                            for (key, value) in appended {
                                if let Some((_, existing)) =
                                    entries.iter_mut().find(|(existing, _)| existing == &key)
                                {
                                    *existing = value;
                                } else {
                                    entries.push((key, value));
                                }
                            }
                            RuntimeValue::Map(entries)
                        }
                        (RuntimeValue::Map(_), _) => {
                            return self.fail("map merge requires another map");
                        }
                        _ => return self.fail("collection append requires a list or map value"),
                    };
                    self.stack.push(result);
                    self.ip += 1;
                }
                Instruction::Print => {
                    let value = self.pop("print")?;
                    self.output.push(format_runtime_value(&value));
                    self.ip += 1;
                }
                Instruction::Pop => {
                    self.pop("discard call result")?;
                    self.ip += 1;
                }
                Instruction::Call { target, arity } => {
                    self.validate_jump(target)?;
                    if self.stack.len() < arity {
                        return self.fail("not enough arguments on the stack for function call");
                    }
                    self.frames.push(CallFrame {
                        return_address: Some(self.ip + 1),
                        stack_base: self.stack.len() - arity,
                        locals: HashMap::new(),
                    });
                    self.ip = target;
                }
                Instruction::Return => {
                    let value = self.pop("return")?;
                    if self.frames.len() == 1 {
                        return self.fail("return executed outside a function");
                    }
                    let frame = self.frames.pop().expect("call frame exists");
                    self.stack.truncate(frame.stack_base);
                    self.stack.push(value);
                    self.ip = frame
                        .return_address
                        .expect("function frame has return address");
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
    fn binary_number(
        &mut self,
        operation: &'static str,
        apply_integer: impl FnOnce(BigInt, BigInt) -> BigInt,
        apply_float: impl FnOnce(f64, f64) -> f64,
    ) -> Result<(), RunError> {
        let right = self.pop_number(operation)?;
        let left = self.pop_number(operation)?;
        let result = match (left, right) {
            (NumberValue::Integer(left), NumberValue::Integer(right)) => {
                RuntimeValue::Num(apply_integer(left, right))
            }
            (left, right) => {
                let (left, right) = self.numbers_as_floats(left, right, operation)?;
                let result = apply_float(left, right);
                self.checked_float(result, operation)?
            }
        };
        self.stack.push(result);
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
            (RuntimeValue::Float(left), RuntimeValue::Float(right)) => {
                let result = self.checked_float(left + right, "add")?;
                self.stack.push(result);
            }
            (RuntimeValue::Num(left), RuntimeValue::Float(right)) => {
                let left = self.integer_as_float(&left, "add")?;
                let result = self.checked_float(left + right, "add")?;
                self.stack.push(result);
            }
            (RuntimeValue::Float(left), RuntimeValue::Num(right)) => {
                let right = self.integer_as_float(&right, "add")?;
                let result = self.checked_float(left + right, "add")?;
                self.stack.push(result);
            }
            (RuntimeValue::Str(left), RuntimeValue::Str(right)) => {
                self.stack
                    .push(RuntimeValue::Str(format!("{}{}", left, right)));
            }
            _ => return self.fail("add requires matching numeric or text values"),
        }
        self.ip += 1;
        Ok(())
    }

    /// Apply a numeric comparison and push the resulting boolean.
    fn compare_num(
        &mut self,
        operation: &'static str,
        apply_integer: impl FnOnce(BigInt, BigInt) -> bool,
        apply_float: impl FnOnce(f64, f64) -> bool,
    ) -> Result<(), RunError> {
        let right = self.pop_number(operation)?;
        let left = self.pop_number(operation)?;
        let result = match (left, right) {
            (NumberValue::Integer(left), NumberValue::Integer(right)) => apply_integer(left, right),
            (left, right) => {
                let (left, right) = self.numbers_as_floats(left, right, operation)?;
                apply_float(left, right)
            }
        };
        self.stack.push(RuntimeValue::Bool(result));
        self.ip += 1;
        Ok(())
    }

    /// Compare scalar runtime values for equality or inequality.
    fn equality(&mut self, invert: bool) -> Result<(), RunError> {
        let right = self.pop("compare equality")?;
        let left = self.pop("compare equality")?;
        let equal = match (left, right) {
            (RuntimeValue::Num(left), RuntimeValue::Num(right)) => left == right,
            (RuntimeValue::Float(left), RuntimeValue::Float(right)) => left == right,
            (RuntimeValue::Num(left), RuntimeValue::Float(right)) => {
                self.integer_as_float(&left, "compare equality")? == right
            }
            (RuntimeValue::Float(left), RuntimeValue::Num(right)) => {
                left == self.integer_as_float(&right, "compare equality")?
            }
            (RuntimeValue::Str(left), RuntimeValue::Str(right)) => left == right,
            (RuntimeValue::Bool(left), RuntimeValue::Bool(right)) => left == right,
            (RuntimeValue::Emoji(left), RuntimeValue::Emoji(right)) => left == right,
            _ => return self.fail("runtime equality requires matching scalar values"),
        };
        self.stack
            .push(RuntimeValue::Bool(if invert { !equal } else { equal }));
        self.ip += 1;
        Ok(())
    }

    /// Pop one value from the operand stack.
    fn pop(&mut self, operation: &'static str) -> Result<RuntimeValue, RunError> {
        self.stack
            .pop()
            .ok_or_else(|| self.error(format!("stack underflow during {}", operation)))
    }

    /// Pop either numeric runtime representation.
    fn pop_number(&mut self, operation: &'static str) -> Result<NumberValue, RunError> {
        match self.pop(operation)? {
            RuntimeValue::Num(value) => Ok(NumberValue::Integer(value)),
            RuntimeValue::Float(value) => Ok(NumberValue::Float(value)),
            _ => Err(self.error(format!("{} requires a numeric value", operation))),
        }
    }

    /// Divide integers exactly as before, or promote a mixed operation to float.
    fn divide_numbers(&mut self) -> Result<(), RunError> {
        let right = self.pop_number("divide")?;
        let left = self.pop_number("divide")?;
        let result = match (left, right) {
            (NumberValue::Integer(left), NumberValue::Integer(right)) => {
                if right.is_zero() {
                    return self.fail("division by zero");
                }
                RuntimeValue::Num(left / right)
            }
            (left, right) => {
                let (left, right) = self.numbers_as_floats(left, right, "divide")?;
                if right == 0.0 {
                    return self.fail("division by zero");
                }
                self.checked_float(left / right, "divide")?
            }
        };
        self.stack.push(result);
        self.ip += 1;
        Ok(())
    }

    fn numbers_as_floats(
        &self,
        left: NumberValue,
        right: NumberValue,
        operation: &'static str,
    ) -> Result<(f64, f64), RunError> {
        Ok((
            self.number_as_float(left, operation)?,
            self.number_as_float(right, operation)?,
        ))
    }

    fn number_as_float(
        &self,
        value: NumberValue,
        operation: &'static str,
    ) -> Result<f64, RunError> {
        match value {
            NumberValue::Integer(value) => self.integer_as_float(&value, operation),
            NumberValue::Float(value) => Ok(value),
        }
    }

    fn integer_as_float(&self, value: &BigInt, operation: &'static str) -> Result<f64, RunError> {
        let Some(converted) = value.to_f64() else {
            return Err(self.error(format!(
                "{} cannot convert this integer to a finite float",
                operation
            )));
        };
        if BigInt::from_f64(converted).as_ref() == Some(value) {
            Ok(converted)
        } else {
            Err(self.error(format!(
                "{} cannot represent this integer exactly as a float",
                operation
            )))
        }
    }

    fn checked_float(&self, value: f64, operation: &'static str) -> Result<RuntimeValue, RunError> {
        if value.is_finite() {
            Ok(RuntimeValue::Float(value))
        } else {
            Err(self.error(format!("{} produced a non-finite float", operation)))
        }
    }

    fn parse_input(&self, kind: InputKind, raw: String) -> Result<RuntimeValue, RunError> {
        match kind {
            InputKind::Text => Ok(RuntimeValue::Str(raw)),
            InputKind::Integer => raw
                .trim()
                .parse::<BigInt>()
                .map(RuntimeValue::Num)
                .map_err(|_| self.error("input is not a valid integer")),
            InputKind::Float => {
                let value = raw
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| self.error("input is not a valid float"))?;
                if value.is_finite() {
                    Ok(RuntimeValue::Float(value))
                } else {
                    Err(self.error("float input must be finite"))
                }
            }
            InputKind::Bool => match raw.trim() {
                "✅" | "true" => Ok(RuntimeValue::Bool(true)),
                "❌" | "false" => Ok(RuntimeValue::Bool(false)),
                _ => Err(self.error("boolean input must be ✅, ❌, true, or false")),
            },
        }
    }

    fn convert_value(&mut self, kind: ConversionKind) -> Result<(), RunError> {
        let value = self.pop("convert")?;
        let converted = match (kind, value) {
            (ConversionKind::Integer, RuntimeValue::Str(value)) => value
                .trim()
                .parse::<BigInt>()
                .map(RuntimeValue::Num)
                .map_err(|_| self.error("text is not a valid integer"))?,
            (ConversionKind::Float, RuntimeValue::Str(value)) => {
                let converted = value
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| self.error("text is not a valid float"))?;
                if !converted.is_finite() {
                    return self.fail("float conversion must produce a finite value");
                }
                RuntimeValue::Float(converted)
            }
            (ConversionKind::Float, RuntimeValue::Num(value)) => {
                let Some(converted) = value.to_f64().filter(|value| value.is_finite()) else {
                    return self.fail("integer is too large to convert to a finite float");
                };
                RuntimeValue::Float(converted)
            }
            (ConversionKind::Integer, _) => {
                return self.fail("integer conversion requires text");
            }
            (ConversionKind::Float, _) => {
                return self.fail("float conversion requires text or an integer");
            }
        };
        self.stack.push(converted);
        self.ip += 1;
        Ok(())
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
        let span = self
            .source_spans
            .and_then(|source_spans| source_spans.get(self.ip))
            .copied()
            .flatten();
        RunError {
            output: self.output.clone(),
            diagnostics: vec![Diagnostic {
                message: message.into(),
                span,
            }],
        }
    }
}

fn same_runtime_type(left: &RuntimeValue, right: &RuntimeValue) -> bool {
    matches!(
        (left, right),
        (RuntimeValue::Num(_), RuntimeValue::Num(_))
            | (RuntimeValue::Float(_), RuntimeValue::Float(_))
            | (RuntimeValue::Str(_), RuntimeValue::Str(_))
            | (RuntimeValue::Bool(_), RuntimeValue::Bool(_))
            | (RuntimeValue::Emoji(_), RuntimeValue::Emoji(_))
            | (RuntimeValue::List(_), RuntimeValue::List(_))
            | (RuntimeValue::Map(_), RuntimeValue::Map(_))
    )
}

impl From<Value> for RuntimeValue {
    /// Convert a bytecode constant into its runtime representation.
    fn from(value: Value) -> Self {
        match value {
            Value::Num(value) => RuntimeValue::Num(value),
            Value::Float(value) => RuntimeValue::Float(value),
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
        RuntimeValue::Float(value) => value.to_string(),
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
        RuntimeValue::Map(entries) => {
            let items = entries
                .iter()
                .map(|(key, value)| format!("💬{}💬 ➡️ {}", key, format_runtime_value(value)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("🗺️ {} 🗺️", items)
        }
    }
}
