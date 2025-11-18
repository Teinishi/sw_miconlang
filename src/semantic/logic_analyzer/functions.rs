use super::LogicAnalyzer;
use crate::{
    compile_error::CompileErrorType,
    microcontroller::{ArithmeticComponent, Component, Link},
    syntax::{Expr, Spanned},
};

fn arg1(args: &[Spanned<Expr>]) -> Result<&Spanned<Expr>, CompileErrorType> {
    if args.len() != 1 {
        //Err(CompileErrorType::InvalidArguments)
        todo!()
    } else {
        Ok(args.iter().next().unwrap())
    }
}

fn arg1_typed<T>(
    logic_analyzer: &mut LogicAnalyzer,
    args: &[Spanned<Expr>],
) -> Option<Result<T, CompileErrorType>>
where
    T: TryFrom<Link>,
    <T as TryFrom<Link>>::Error: Into<CompileErrorType>,
{
    match arg1(args).map(|i| logic_analyzer.expr_to_component_typed(i)) {
        Ok(Some(r)) => Some(Ok(r)),
        Ok(None) => None,
        Err(err) => Some(Err(err)),
    }
}

fn arg2(args: &[Spanned<Expr>]) -> Result<(&Spanned<Expr>, &Spanned<Expr>), CompileErrorType> {
    if args.len() != 2 {
        //Err(CompileErrorType::InvalidArguments)
        todo!()
    } else {
        let mut iter = args.iter();
        Ok((iter.next().unwrap(), iter.next().unwrap()))
    }
}

fn arg2_typed<T0, T1>(
    logic_analyzer: &mut LogicAnalyzer,
    args: &[Spanned<Expr>],
) -> Option<Result<(T0, T1), CompileErrorType>>
where
    T0: TryFrom<Link>,
    <T0 as TryFrom<Link>>::Error: Into<CompileErrorType>,
    T1: TryFrom<Link>,
    <T1 as TryFrom<Link>>::Error: Into<CompileErrorType>,
{
    match arg2(args).map(|(i0, i1)| {
        (logic_analyzer
            .expr_to_component_typed(i0)
            .zip(logic_analyzer.expr_to_component_typed(i1)))
    }) {
        Ok(Some(r)) => Some(Ok(r)),
        Ok(None) => None,
        Err(err) => Some(Err(err)),
    }
}

pub(super) fn function_calls(
    logic_analyzer: &mut LogicAnalyzer,
    name: &str,
    args: &[Spanned<Expr>],
) -> Option<Result<Link, CompileErrorType>> {
    let r = match name {
        "abs" => arg1_typed(logic_analyzer, args).map(|i| {
            i.map(|i| {
                (
                    Component::Arithmetic(ArithmeticComponent::Abs { input: i }),
                    0,
                )
            })
        }),
        _ => todo!(),
    };

    match r {
        Some(Ok((component, index))) => Some(logic_analyzer.add_component(component, index)),
        Some(Err(err)) => Some(Err(err)),
        None => None,
    }
}
