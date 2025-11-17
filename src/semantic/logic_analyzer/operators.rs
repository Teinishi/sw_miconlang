use super::LogicAnalyzer;
use crate::{
    compile_error::CompileErrorType,
    microcontroller::{ArithmeticComponent, Component, Link},
    syntax::{BinaryOp, UnaryOp},
};

pub(super) fn binary_operation(
    logic_analyzer: &mut LogicAnalyzer,
    op: &BinaryOp,
) -> Option<Result<Link, CompileErrorType>> {
    let (component, index) = match op {
        BinaryOp::Add(lhs, rhs) => (
            Component::Arithmetic(ArithmeticComponent::Add {
                input_a: logic_analyzer.expr_to_component_typed(lhs)?,
                input_b: logic_analyzer.expr_to_component_typed(rhs)?,
            }),
            0,
        ),
        BinaryOp::Sub(lhs, rhs) => (
            Component::Arithmetic(ArithmeticComponent::Subtract {
                input_a: logic_analyzer.expr_to_component_typed(lhs)?,
                input_b: logic_analyzer.expr_to_component_typed(rhs)?,
            }),
            0,
        ),
        BinaryOp::Mul(lhs, rhs) => (
            Component::Arithmetic(ArithmeticComponent::Multiply {
                input_a: logic_analyzer.expr_to_component_typed(lhs)?,
                input_b: logic_analyzer.expr_to_component_typed(rhs)?,
            }),
            0,
        ),
        BinaryOp::Div(lhs, rhs) => (
            Component::Arithmetic(ArithmeticComponent::Divide {
                input_a: logic_analyzer.expr_to_component_typed(lhs)?,
                input_b: logic_analyzer.expr_to_component_typed(rhs)?,
            }),
            0,
        ),
    };
    Some(logic_analyzer.add_component(component, index))
}

pub(super) fn unary_operation(
    logic_analyzer: &mut LogicAnalyzer,
    op: &UnaryOp,
) -> Option<Result<Link, CompileErrorType>> {
    let (component, index) = match op {
        UnaryOp::Neg(x) => (
            Component::Arithmetic(ArithmeticComponent::Function1 {
                input_x: logic_analyzer.expr_to_component_typed(x)?,
                function: "-x".to_owned(),
            }),
            0,
        ),
    };
    Some(logic_analyzer.add_component(component, index))
}
