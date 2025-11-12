use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{Microcontroller, UnpositionedMicrocontroller},
    syntax::{self, Assignment, Expr, LiteralValue, Spanned},
};

use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Debug)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
}

impl From<&LiteralValue> for ValueType {
    fn from(value: &LiteralValue) -> Self {
        match value {
            LiteralValue::Bool(_) => Self::Bool,
            LiteralValue::Int(_) => Self::Int,
            LiteralValue::Float(_) => Self::Float,
            LiteralValue::String(_) => Self::String,
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
        }
    }
}

#[derive(Debug)]
pub struct FileAnalyzeResult<'a, 'b> {
    microcontrollers: HashMap<String, UnpositionedMicrocontroller>,
    errors: Vec<CompileError<'a, 'b>>,
}

impl<'a, 'b> FileAnalyzeResult<'a, 'b> {
    pub fn into_output(self) -> Option<HashMap<String, UnpositionedMicrocontroller>> {
        if self.has_errors() {
            None
        } else {
            Some(self.microcontrollers)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<CompileError<'a, 'b>> {
        &self.errors
    }
}

pub fn analyze_file<'a, 'b>(
    tree: &'b Spanned<syntax::File>,
    filename: &'a str,
) -> FileAnalyzeResult<'a, 'b> {
    let mut microcontrollers = HashMap::new();
    let mut errors = Vec::new();

    for element in &tree.elements {
        match &element.inner {
            syntax::Element::Microcontroller { name, elements } => {
                if let Some(mc) = analyze_microcontroller(elements, filename, &mut errors) {
                    microcontrollers.insert(name.clone(), mc);
                }
            }
        }
    }

    FileAnalyzeResult {
        microcontrollers,
        errors,
    }
}

fn analyze_microcontroller<'a, 'b>(
    elements: &'b [Spanned<syntax::MicrocontrollerElement>],
    filename: &'a str,
    errors: &mut Vec<CompileError<'a, 'b>>,
) -> Option<UnpositionedMicrocontroller> {
    let mut mc = Microcontroller::default();

    for element in elements {
        match &element.inner {
            syntax::MicrocontrollerElement::Field(assignment) => {
                if let Err(err) = analyze_microcontroller_field(assignment, &mut mc, filename) {
                    errors.push(err);
                }
            }
            syntax::MicrocontrollerElement::Interface(interfaces) => {}
            syntax::MicrocontrollerElement::Logic(statements) => {}
        }
    }

    Some(mc)
}

fn analyze_microcontroller_field<'a, 'b>(
    assignment: &'b Spanned<Assignment>,
    mc: &mut UnpositionedMicrocontroller,
    filename: &'a str,
) -> Result<(), CompileError<'a, 'b>> {
    let target = &assignment.target.inner;
    let target_span = &assignment.target.span;
    let value = &assignment.value.inner;
    let value_span = &assignment.value.span;

    let mut mut_target = if let Expr::Ident(ident) = target {
        match ident.as_str() {
            "name" => MutField::String(&mut mc.name),
            "description" => MutField::String(&mut mc.description),
            "width" => MutField::RangedU8(&mut mc.width, 1..=6),
            "length" => MutField::RangedU8(&mut mc.length, 1..=6),
            _ => {
                return Err(CompileError::new(
                    filename,
                    target_span.clone(),
                    CompileErrorType::UnknownField { ident },
                ));
            }
        }
    } else {
        return Err(CompileError::new(
            filename,
            target_span.clone(),
            CompileErrorType::InvalidAssignment,
        ));
    };

    if let Expr::LiteralValue(value) = value {
        mut_target
            .assign_literal(value)
            .map_err(|err| CompileError::new(filename, value_span.clone(), err))?;
    } else {
        return Err(CompileError::new(
            filename,
            value_span.clone(),
            CompileErrorType::LiteralExpected,
        ));
    }

    Ok(())
}

#[derive(Debug)]
enum MutField<'a> {
    Bool(&'a mut bool),
    RangedU8(&'a mut u8, RangeInclusive<i64>),
    String(&'a mut String),
}

impl<'a> MutField<'a> {
    fn expected_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::RangedU8(_, _) => ValueType::Int,
            Self::String(_) => ValueType::String,
        }
    }

    fn assign_literal<'b>(&'a mut self, value: &LiteralValue) -> Result<(), CompileErrorType<'b>> {
        let expected_type = self.expected_type();
        match (self, value) {
            (Self::Bool(p), LiteralValue::Bool(v)) => {
                **p = *v;
            }
            (Self::RangedU8(p, range), LiteralValue::Int(v)) => {
                if !range.contains(v) {
                    return Err(CompileErrorType::OutOfBounds {
                        bounds: range.clone(),
                    });
                }
                **p = (*v)
                    .try_into()
                    .map_err(|_| CompileErrorType::OutOfBounds { bounds: 0..=255 })?;
            }
            (Self::String(p), LiteralValue::String(v)) => {
                **p = v.clone();
            }
            _ => {
                return Err(CompileErrorType::IncompatibleTypes {
                    expected_type,
                    found_type: value.into(),
                });
            }
        }

        Ok(())
    }
}
