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

pub(super) fn function_calls(
    logic_analyzer: &mut LogicAnalyzer,
    name: &str,
    args: &[Spanned<Expr>],
) -> Option<Result<Link, CompileErrorType>> {
    let (component, index) = match name {
        "abs" => match arg1(args) {
            Ok(i) => (
                Component::Arithmetic(ArithmeticComponent::Abs {
                    input: logic_analyzer.expr_to_component_typed(i)?,
                }),
                0,
            ),
            Err(err) => {
                return Some(Err(err));
            }
        },
        _ => todo!(),
    };

    Some(logic_analyzer.add_component(component, index))
}
