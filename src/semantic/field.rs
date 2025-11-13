use super::ValueType;
use crate::{
    compile_error::{CompileError, CompileErrorType},
    syntax::{Assignment, Expr, Spanned},
};

use std::ops::RangeInclusive;

pub(super) fn analyze_field<'a, 'm, F>(
    assignment: &Spanned<Assignment>,
    filename: &'a str,
    mut_field: F,
) -> Result<(), CompileError<'a>>
where
    F: FnOnce(&String) -> Option<MutField<'m>>,
{
    let target = &assignment.target.inner;
    let target_span = &assignment.target.span;

    let mut_target = if let Expr::Ident(ident) = target {
        if let Some(t) = mut_field(ident) {
            t
        } else {
            return Err(CompileError::new(
                filename,
                assignment.target.span.clone(),
                CompileErrorType::UnknownField {
                    ident: ident.to_owned(),
                },
            ));
        }
    } else {
        return Err(CompileError::new(
            filename,
            target_span.clone(),
            CompileErrorType::InvalidAssignment,
        ));
    };

    // todo: きちんと expr を評価して assign できるようにする
    mut_target.assign(&assignment.value, filename)?;

    Ok(())
}

#[derive(Debug)]
pub(super) enum MutField<'m> {
    #[expect(dead_code)]
    Bool(&'m mut Option<bool>),
    RangedU8(&'m mut Option<u8>, RangeInclusive<i64>),
    String(&'m mut Option<String>),
    TupleTwoRangedU8(
        &'m mut Option<(u8, u8)>,
        (RangeInclusive<i64>, RangeInclusive<i64>),
    ),
}

impl<'m> MutField<'m> {
    fn assign<'a>(self, value: &Spanned<Expr>, filename: &'a str) -> Result<(), CompileError<'a>> {
        match self {
            Self::Bool(target) => assign_bool(target, value, filename),
            Self::RangedU8(target, range) => assign_ranged_u8(target, &range, value, filename),
            Self::String(target) => assign_string(target, value, filename),
            Self::TupleTwoRangedU8(target, ranges) => {
                assign_tuple_two_ranged_u8(target, &ranges, value, filename)
            }
        }
    }
}

fn assign_bool<'a>(
    target: &mut Option<bool>,
    value: &Spanned<Expr>,
    filename: &'a str,
) -> Result<(), CompileError<'a>> {
    if let Expr::BoolLiteral(v) = &value.inner {
        *target = Some(*v);
        Ok(())
    } else {
        Err(CompileError::new(
            filename,
            value.span.clone(),
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::Bool],
                found_type: ValueType::from_expr(value),
            },
        ))
    }
}

fn assign_ranged_u8<'a>(
    target: &mut Option<u8>,
    range: &RangeInclusive<i64>,
    value: &Spanned<Expr>,
    filename: &'a str,
) -> Result<(), CompileError<'a>> {
    if let Expr::IntLiteral(v) = &value.inner {
        if !range.contains(v) {
            return Err(CompileError::new(
                filename,
                value.span.clone(),
                CompileErrorType::OutOfBounds {
                    bounds: range.clone(),
                },
            ));
        }
        let v = (*v).try_into().map_err(|_| {
            CompileError::new(
                filename,
                value.span.clone(),
                CompileErrorType::OutOfBounds { bounds: 0..=255 },
            )
        })?;
        *target = Some(v);
        Ok(())
    } else {
        Err(CompileError::new(
            filename,
            value.span.clone(),
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::Int],
                found_type: ValueType::from_expr(value),
            },
        ))
    }
}

fn assign_string<'a>(
    target: &mut Option<String>,
    value: &Spanned<Expr>,
    filename: &'a str,
) -> Result<(), CompileError<'a>> {
    if let Expr::StringLiteral(v) = &value.inner {
        *target = Some(v.clone());
        Ok(())
    } else {
        Err(CompileError::new(
            filename,
            value.span.clone(),
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::String],
                found_type: ValueType::from_expr(value),
            },
        ))
    }
}

fn assign_tuple_two_ranged_u8<'a>(
    target: &mut Option<(u8, u8)>,
    ranges: &(RangeInclusive<i64>, RangeInclusive<i64>),
    value: &Spanned<Expr>,
    filename: &'a str,
) -> Result<(), CompileError<'a>> {
    if let Expr::Tuple(values) = &value.inner {
        let (v0, v1) = try_to_tuple(values).ok_or_else(|| {
            CompileError::new(
                filename,
                value.span.clone(),
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Tuple(vec![ValueType::Int, ValueType::Int])],
                    found_type: ValueType::from_expr(&value.inner),
                },
            )
        })?;

        let mut t0 = None;
        let mut t1 = None;
        assign_ranged_u8(&mut t0, &ranges.0, v0, filename)?;
        assign_ranged_u8(&mut t1, &ranges.1, v1, filename)?;
        *target = Some((t0.unwrap(), t1.unwrap()));

        Ok(())
    } else {
        Err(CompileError::new(
            filename,
            value.span.clone(),
            CompileErrorType::IncompatibleType {
                expected_types: vec![ValueType::Tuple(vec![ValueType::Int, ValueType::Int])],
                found_type: ValueType::from_expr(value),
            },
        ))
    }
}

fn try_to_tuple<T>(v: &[T]) -> Option<(&T, &T)> {
    match std::convert::TryInto::<&[T; 2]>::try_into(v) {
        Ok([a, b]) => Some((a, b)),
        Err(_) => None,
    }
}
