use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{ArithmeticComponent, Component, InputNode, Link, OutputNode},
    syntax::{AssignmentTarget, BinaryOp, Expr, Spanned, Statement, UnaryOp},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

type Span = std::ops::Range<usize>;

#[derive(Default, Debug)]
struct Context {
    variables: HashMap<String, Link>,
}

impl Context {
    fn define_variable(&mut self, ident: String, link: Link) {
        self.variables.insert(ident, link);
    }

    fn get_variable(&self, ident: &String) -> Result<Link, CompileErrorType> {
        if let Some(link) = self.variables.get(ident) {
            Ok(link.clone())
        } else {
            Err(CompileErrorType::UnknownName {
                name: ident.clone(),
            })
        }
    }
}

#[derive(Debug)]
pub(super) struct LogicAnalyzer<'a> {
    filename: &'a str,
    inputs: HashMap<String, Rc<InputNode>>,
    outputs: HashMap<String, Rc<RefCell<OutputNode>>>,
    context: Context,
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
            context: Context::default(),
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

                    if let Err(err) = self.assign_link(target, link, &value.span) {
                        errors.push(err);
                    }
                }
                Statement::Let(ident, value) => {
                    let link = self.expr_to_components(value);
                    if let Err(err) = link {
                        errors.push(err);
                        continue;
                    }
                    self.context.define_variable(ident.clone(), link.unwrap());
                }
            }
        }
    }

    fn add_component(
        &mut self,
        component: Component,
        index: usize,
        span: &Span,
    ) -> Result<Link, CompileError<'a>> {
        let rc = Rc::new(component);
        if let Some(link) = Link::component(&rc, index) {
            self.components.push(rc);
            Ok(link)
        } else {
            Err(CompileError::new(
                self.filename,
                span.clone(),
                CompileErrorType::NodeDoesNotExist {
                    component_str: format!("{}", rc),
                    index,
                },
            ))
        }
    }

    fn expr_to_components(&mut self, expr: &Spanned<Expr>) -> Result<Link, CompileError<'a>> {
        let l = match &expr.inner {
            Expr::BoolLiteral(_) => todo!(),
            Expr::IntLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
                &expr.span,
            )?,
            Expr::FloatLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
                &expr.span,
            )?,
            Expr::StringLiteral(_) => {
                return Err(CompileError::new(
                    self.filename,
                    expr.span.clone(),
                    CompileErrorType::StringInLogic,
                ));
            }
            Expr::Ident(ident) => self
                .context
                .get_variable(ident)
                .map_err(|err| CompileError::new(self.filename, expr.span.clone(), err))?,
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
                            input_a: self.expr_to_component_typed(lhs)?,
                            input_b: self.expr_to_component_typed(rhs)?,
                        }),
                        0,
                    ),
                    BinaryOp::Sub(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Subtract {
                            input_a: self.expr_to_component_typed(lhs)?,
                            input_b: self.expr_to_component_typed(rhs)?,
                        }),
                        0,
                    ),
                    BinaryOp::Mul(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Multiply {
                            input_a: self.expr_to_component_typed(lhs)?,
                            input_b: self.expr_to_component_typed(rhs)?,
                        }),
                        0,
                    ),
                    BinaryOp::Div(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Divide {
                            input_a: self.expr_to_component_typed(lhs)?,
                            input_b: self.expr_to_component_typed(rhs)?,
                        }),
                        0,
                    ),
                };
                self.add_component(component, index, &expr.span)?
            }
            Expr::UnaryOp(op) => {
                let (component, index) = match op {
                    UnaryOp::Neg(x) => (
                        Component::Arithmetic(ArithmeticComponent::Function1 {
                            input_x: self.expr_to_component_typed(x)?,
                            function: "-x".to_owned(),
                        }),
                        0,
                    ),
                };
                self.add_component(component, index, &expr.span)?
            }
        };

        Ok(l)
    }

    fn expr_to_component_typed<T>(&mut self, expr: &Spanned<Expr>) -> Result<T, CompileError<'a>>
    where
        T: TryFrom<Link>,
        <T as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        T::try_from(self.expr_to_components(expr)?)
            .map_err(|err| CompileError::new(self.filename, expr.span.clone(), err.into()))
    }

    fn assign_link(
        &mut self,
        target: &Spanned<AssignmentTarget>,
        link: Link,
        value_span: &Span,
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
                        let found_type = link.node_type();
                        if !o.borrow_mut().set_input_link(link) {
                            // ノードの型エラー
                            return Err(CompileError::new(
                                self.filename,
                                value_span.clone(),
                                CompileErrorType::IncompatibleNodeType {
                                    expected_type: o.borrow().node_type(),
                                    found_type,
                                },
                            ));
                        }
                    }
                    AssignmentTarget::FieldAccess(_, _) => todo!(),
                };
            }
        }

        Ok(())
    }
}
