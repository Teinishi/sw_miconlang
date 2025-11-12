use std::collections::HashMap;

use crate::{
    microcontroller::{Microcontroller, UnpositionedMicrocontroller},
    syntax::{self, Assignment, Expr, LiteralValue, Spanned},
};

pub type Span = std::ops::Range<usize>;

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
            LiteralValue::Number(_) => Self::Float,
            LiteralValue::String(_) => Self::String,
        }
    }
}

#[derive(Debug)]
pub enum SemanticError<'a> {
    UnknownField {
        ident: &'a str,
        span: Span,
    },
    InvalidAssignment {
        expr: &'a Expr,
        span: Span,
    },
    IncompatibleTypes {
        expected_type: ValueType,
        found_type: ValueType,
        expr: &'a Expr,
        span: Span,
    },
    LiteralExpected {
        expr: &'a Expr,
        span: Span,
    },
}

pub fn analyze_file<'a>(
    tree: &'a Spanned<syntax::File>,
) -> Result<HashMap<String, UnpositionedMicrocontroller>, SemanticError<'a>> {
    let mut microcontrollers = HashMap::new();
    for element in &tree.elements {
        match &element.inner {
            syntax::Element::Microcontroller { name, elements } => {
                microcontrollers.insert(name.clone(), analyze_microcontroller(elements)?);
            }
        }
    }

    Ok(microcontrollers)
}

fn analyze_microcontroller<'a>(
    elements: &'a [Spanned<syntax::MicrocontrollerElement>],
) -> Result<UnpositionedMicrocontroller, SemanticError<'a>> {
    let mut mc = Microcontroller::default();

    for element in elements {
        match &element.inner {
            syntax::MicrocontrollerElement::Field(assignment) => {
                analyze_microcontroller_field(assignment, &mut mc)?;
            }
            syntax::MicrocontrollerElement::Interface(interfaces) => {}
            syntax::MicrocontrollerElement::Logic(statements) => {}
        }
    }

    Ok(mc)
}

fn analyze_microcontroller_field<'a>(
    assignment: &'a Spanned<Assignment>,
    mc: &'_ mut UnpositionedMicrocontroller,
) -> Result<(), SemanticError<'a>> {
    dbg!(&assignment);
    let target = &assignment.target.inner;
    let value = &assignment.value.inner;

    let mut mut_target = if let Expr::Ident(ident) = target {
        match ident.as_str() {
            "name" => MutField::String(&mut mc.name),
            "description" => MutField::String(&mut mc.description),
            "width" => MutField::U8(&mut mc.width),
            "length" => MutField::U8(&mut mc.length),
            _ => {
                return Err(SemanticError::UnknownField {
                    ident,
                    span: assignment.target.span.clone(),
                });
            }
        }
    } else {
        return Err(SemanticError::InvalidAssignment {
            expr: target,
            span: assignment.target.span.clone(),
        });
    };

    if let Expr::LiteralValue(value) = value {
        mut_target.assign_literal(value, &assignment.value)?;
    } else {
        return Err(SemanticError::LiteralExpected {
            expr: value,
            span: assignment.value.span.clone(),
        });
    }

    Ok(())
}

#[derive(Debug)]
enum MutField<'a> {
    Bool(&'a mut bool),
    U8(&'a mut u8),
    String(&'a mut String),
}

impl<'a> MutField<'a> {
    fn expected_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::U8(_) => ValueType::Int,
            Self::String(_) => ValueType::String,
        }
    }

    fn assign_literal<'b>(
        &'a mut self,
        value: &LiteralValue,
        spanned_expr: &'b Spanned<Expr>,
    ) -> Result<(), SemanticError<'b>> {
        let expected_type = self.expected_type();
        match (self, value) {
            (Self::Bool(p), LiteralValue::Bool(v)) => {
                **p = *v;
            }
            (Self::U8(p), LiteralValue::Number(v)) => {
                todo!()
            }
            (Self::String(p), LiteralValue::String(v)) => {
                **p = v.clone();
            }
            _ => {
                return Err(SemanticError::IncompatibleTypes {
                    expected_type,
                    found_type: value.into(),
                    expr: &spanned_expr.inner,
                    span: spanned_expr.span.clone(),
                });
            }
        }

        Ok(())
    }
}
