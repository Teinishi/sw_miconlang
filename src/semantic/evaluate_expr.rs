use super::ValueType;
use crate::{
    compile_error::{CompileError, CompileErrorType},
    syntax::{Expr, Spanned},
};

use std::ops::{Range, RangeInclusive};

fn to_tuple_2<T>(values: Vec<T>) -> Option<(T, T)> {
    let mut iter = values.into_iter();
    let v0 = iter.next()?;
    let v1 = iter.next()?;
    Some((v0, v1))
}

pub(super) fn evaluate_expr<'a>(
    spanned_expr: &Spanned<Expr>,
    filename: &'a str,
) -> Result<EvaluatedValue<'a>, CompileError<'a>> {
    let inner = match &spanned_expr.inner {
        Expr::BoolLiteral(v) => EvaluatedValueInner::Bool(*v),
        Expr::IntLiteral(v) => EvaluatedValueInner::Int(*v),
        Expr::FloatLiteral(v) => EvaluatedValueInner::Float(*v),
        Expr::StringLiteral(v) => EvaluatedValueInner::String(v.clone()),
        Expr::Ident(_) => todo!(),
        Expr::Inputs => todo!(),
        Expr::Outputs => todo!(),
        Expr::Tuple(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(evaluate_expr(item, filename)?);
            }
            EvaluatedValueInner::Tuple(values)
        }
        Expr::FieldAccess(_, _) => todo!(),
        Expr::BinaryOp(_) => todo!(),
        Expr::UnaryOp(_) => todo!(),
    };
    Ok(EvaluatedValue {
        inner,
        filename,
        span: spanned_expr.span.clone(),
    })
}

#[derive(Debug)]
enum EvaluatedValueInner<'a> {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Tuple(Vec<EvaluatedValue<'a>>),
}

impl<'a> EvaluatedValueInner<'a> {
    fn value_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::Int(_) => ValueType::Int,
            Self::Float(_) => ValueType::Float,
            Self::String(_) => ValueType::String,
            Self::Tuple(items) => {
                ValueType::Tuple(items.iter().map(|i| i.inner.value_type()).collect())
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct EvaluatedValue<'a> {
    inner: EvaluatedValueInner<'a>,
    filename: &'a str,
    span: Range<usize>,
}

impl<'a> EvaluatedValue<'a> {
    pub(super) fn int_ranged(self, range: RangeInclusive<i64>) -> Result<Self, CompileError<'a>> {
        match &self.inner {
            EvaluatedValueInner::Int(v) => {
                if range.contains(v) {
                    Ok(self)
                } else {
                    Err(CompileError::new(
                        self.filename,
                        self.span,
                        CompileErrorType::OutOfBounds { bounds: range },
                    ))
                }
            }
            _ => Err(CompileError::new(
                self.filename,
                self.span,
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Int],
                    found_type: self.inner.value_type(),
                },
            )),
        }
    }

    pub(super) fn tuple_int_ranged(
        mut self,
        ranges: Vec<RangeInclusive<i64>>,
    ) -> Result<Self, CompileError<'a>> {
        if let EvaluatedValueInner::Tuple(items) = &mut self.inner
            && items.len() == ranges.len()
        {
            let new_items = std::mem::take(items)
                .into_iter()
                .zip(ranges)
                .map(|(item, range)| item.int_ranged(range))
                .collect::<Result<Vec<_>, _>>()?;
            *items = new_items;
            return Ok(self);
        }
        Err(CompileError::new(
            self.filename,
            self.span,
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::Int],
                found_type: self.inner.value_type(),
            },
        ))
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for bool {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        match value.inner {
            EvaluatedValueInner::Bool(v) => Ok(v),
            _ => Err(CompileError::new(
                value.filename,
                value.span,
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Bool],
                    found_type: value.inner.value_type(),
                },
            )),
        }
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for i64 {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        match value.inner {
            EvaluatedValueInner::Int(v) => Ok(v),
            _ => Err(CompileError::new(
                value.filename,
                value.span,
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Int],
                    found_type: value.inner.value_type(),
                },
            )),
        }
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for f64 {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        match value.inner {
            EvaluatedValueInner::Float(v) => Ok(v),
            _ => Err(CompileError::new(
                value.filename,
                value.span,
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Float],
                    found_type: value.inner.value_type(),
                },
            )),
        }
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for String {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        match value.inner {
            EvaluatedValueInner::String(v) => Ok(v),
            _ => Err(CompileError::new(
                value.filename,
                value.span,
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::String],
                    found_type: value.inner.value_type(),
                },
            )),
        }
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for u8 {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        let filename = value.filename;
        let span = value.span.clone();
        let v = i64::try_from(value)?;
        u8::try_from(v).map_err(|_| {
            CompileError::new(
                filename,
                span,
                CompileErrorType::OutOfBounds { bounds: 0..=255 },
            )
        })
    }
}

impl<'a> TryFrom<EvaluatedValue<'a>> for (u8, u8) {
    type Error = CompileError<'a>;

    fn try_from(value: EvaluatedValue<'a>) -> Result<Self, Self::Error> {
        let found_type = value.inner.value_type();
        if let EvaluatedValueInner::Tuple(items) = value.inner
            && let Some((v0, v1)) = to_tuple_2(items)
        {
            return Ok((v0.try_into()?, v1.try_into()?));
        }
        Err(CompileError::new(
            value.filename,
            value.span,
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::Tuple(vec![ValueType::Int, ValueType::Int])],
                found_type,
            },
        ))
    }
}
