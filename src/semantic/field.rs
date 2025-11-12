use super::ValueType;
use crate::{
    compile_error::{CompileError, CompileErrorType},
    syntax::{Assignment, Expr, LiteralValue, Spanned},
};

use std::ops::RangeInclusive;

pub(super) fn analyze_field<'a, 'b, 'c, F>(
    assignment: &'b Spanned<Assignment>,
    filename: &'a str,
    mut_field: F,
) -> Result<(), CompileError<'a, 'b>>
where
    F: FnOnce(&'b String) -> Option<MutField<'c>>,
{
    let target = &assignment.target.inner;
    let target_span = &assignment.target.span;
    let value = &assignment.value.inner;
    let value_span = &assignment.value.span;

    let mut mut_target = if let Expr::Ident(ident) = target {
        if let Some(t) = mut_field(ident) {
            t
        } else {
            return Err(CompileError::new(
                filename,
                assignment.target.span.clone(),
                CompileErrorType::UnknownField { ident },
            ));
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
pub(super) enum MutField<'a> {
    #[expect(dead_code)]
    Bool(&'a mut bool),
    RangedU8(&'a mut u8, RangeInclusive<i64>),
    String(&'a mut String),
    #[expect(dead_code)]
    TupleTwoRangedU8(&'a mut (u8, u8), (RangeInclusive<i64>, RangeInclusive<i64>)),
}

impl<'a> MutField<'a> {
    fn expected_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::RangedU8(_, _) => ValueType::Int,
            Self::String(_) => ValueType::String,
            Self::TupleTwoRangedU8(_, _) => ValueType::Tuple(vec![ValueType::Int, ValueType::Int]),
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
                return Err(CompileErrorType::IncompatibleType {
                    expected_types: vec![expected_type],
                    found_type: value.into(),
                });
            }
        }

        Ok(())
    }

    // todo: きちんと expr を評価して assign できるようにする
}
