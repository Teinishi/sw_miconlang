use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{ArithmeticComponent, Component, InputNode, Link, OutputNode},
    syntax::{AssignmentTarget, BinaryOp, Expr, Spanned, Statement, UnaryOp},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub(super) struct LogicAnalyzer<'a> {
    filename: &'a str,
    inputs: HashMap<String, Rc<InputNode>>,
    outputs: HashMap<String, Rc<RefCell<OutputNode>>>,
    pub(super) components: Vec<Rc<Component>>,
}

impl<'a> LogicAnalyzer<'a> {
    pub(super) fn new(
        filename: &'a str,
        inputs: HashMap<String, Rc<InputNode>>,
        outputs: HashMap<String, Rc<RefCell<OutputNode>>>,
    ) -> Self {
        Self {
            filename,
            inputs,
            outputs,
            components: Vec::new(),
        }
    }

    pub(super) fn statements(
        &mut self,
        statements: &[Spanned<Statement>],
        errors: &mut Vec<CompileError<'a>>,
    ) {
        for statement in statements {
            match &statement.inner {
                Statement::Assignment(assignment) => {
                    let target = &assignment.inner.target;
                    let value = &assignment.inner.value;

                    let link = self.expr_to_components(value);
                    if let Err(err) = link {
                        errors.push(err);
                        continue;
                    }
                    let link = link.unwrap();

                    if let Err(err) = self.assign_link(target, link) {
                        errors.push(err);
                    }
                }
            }
        }
    }

    fn add_component(&mut self, component: Component, index: usize) -> Link {
        let rc = Rc::new(component);
        let link = Link::component(&rc, index);
        self.components.push(rc);
        link
    }

    fn expr_to_components(&mut self, expr: &Spanned<Expr>) -> Result<Link, CompileError<'a>> {
        let l = match &expr.inner {
            Expr::BoolLiteral(_) => todo!(),
            Expr::IntLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
            ),
            Expr::FloatLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
            ),
            Expr::StringLiteral(_) => {
                return Err(CompileError::new(
                    self.filename,
                    expr.span.clone(),
                    CompileErrorType::StringInLogic,
                ));
            }
            Expr::Ident(_) => todo!(),
            Expr::Inputs => {
                return Err(CompileError::new(
                    self.filename,
                    expr.span.clone(),
                    CompileErrorType::FieldAccessOnly,
                ));
            }
            Expr::Outputs => {
                return Err(CompileError::new(
                    self.filename,
                    expr.span.clone(),
                    CompileErrorType::OutputsInExpression,
                ));
            }
            Expr::Tuple(_) => todo!(),
            Expr::FieldAccess(object, field) => match &object.inner {
                Expr::Inputs => Link::node(self.inputs.get(field).ok_or_else(|| {
                    CompileError::new(
                        self.filename,
                        expr.span.clone(),
                        CompileErrorType::UnknownField {
                            ident: field.to_owned(),
                        },
                    )
                })?),
                _ => todo!(),
            },
            Expr::BinaryOp(op) => {
                let (component, index) = match op {
                    BinaryOp::Add(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Add {
                            input_a: Some(self.expr_to_components(lhs)?),
                            input_b: Some(self.expr_to_components(rhs)?),
                        }),
                        0,
                    ),
                    BinaryOp::Sub(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Subtract {
                            input_a: Some(self.expr_to_components(lhs)?),
                            input_b: Some(self.expr_to_components(rhs)?),
                        }),
                        0,
                    ),
                    BinaryOp::Mul(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Multiply {
                            input_a: Some(self.expr_to_components(lhs)?),
                            input_b: Some(self.expr_to_components(rhs)?),
                        }),
                        0,
                    ),
                    BinaryOp::Div(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Divide {
                            input_a: Some(self.expr_to_components(lhs)?),
                            input_b: Some(self.expr_to_components(rhs)?),
                        }),
                        0,
                    ),
                };
                self.add_component(component, index)
            }
            Expr::UnaryOp(op) => {
                let (component, index) = match op {
                    UnaryOp::Neg(x) => (
                        Component::Arithmetic(ArithmeticComponent::Function1 {
                            input_x: Some(self.expr_to_components(x)?),
                            function: "-x".to_owned(),
                        }),
                        0,
                    ),
                };
                self.add_component(component, index)
            }
        };

        Ok(l)
    }

    fn assign_link(
        &mut self,
        target: &Spanned<AssignmentTarget>,
        link: Link,
    ) -> Result<(), CompileError<'a>> {
        match &target.inner {
            AssignmentTarget::Ident(_) => todo!(),
            AssignmentTarget::Inputs => todo!(),
            AssignmentTarget::Outputs => todo!(),
            AssignmentTarget::FieldAccess(object, field) => {
                match &object.inner {
                    AssignmentTarget::Ident(_) => todo!(),
                    AssignmentTarget::Inputs => todo!(),
                    AssignmentTarget::Outputs => {
                        let o = self.outputs.get_mut(field).ok_or_else(|| {
                            CompileError::new(
                                self.filename,
                                target.span.clone(),
                                CompileErrorType::UnknownField {
                                    ident: field.to_owned(),
                                },
                            )
                        })?;
                        o.borrow_mut().input = Some(link);
                    }
                    AssignmentTarget::FieldAccess(_, _) => todo!(),
                };
            }
        }

        Ok(())
    }
}
