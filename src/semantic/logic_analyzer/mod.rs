mod functions;
mod operators;
use operators::{binary_operation, unary_operation};

use super::evaluate_expr::{EvaluatedValue, evaluate_expr};
use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{ArithmeticComponent, Component, InputNode, Link, OutputNode},
    syntax::{AssignmentTarget, Expr, Spanned, Statement},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

type Span = std::ops::Range<usize>;
type Inputs = HashMap<String, Rc<InputNode>>;
type Outputs = HashMap<String, Rc<RefCell<OutputNode>>>;

#[derive(Debug)]
struct ContextRoot {
    inputs: Inputs,
    outputs: Outputs,
}

#[derive(Default, Debug)]
struct ContextScope {
    variables: HashMap<String, Link>,
}

#[derive(Debug)]
pub(super) struct Context {
    root: Option<ContextRoot>,
    stack: Vec<ContextScope>,
}

impl Context {
    pub(super) fn new(inputs: Inputs, outputs: Outputs) -> Self {
        Self {
            root: Some(ContextRoot { inputs, outputs }),
            stack: Vec::new(),
        }
    }

    fn get_variable(&self, ident: &str) -> Option<Link> {
        for scope in self.stack.iter().rev() {
            if let Some(v) = scope.variables.get(ident) {
                return Some(v.clone());
            }
        }
        None
    }

    fn define_variable(&mut self, ident: String, link: Link) {
        if let Some(scope) = self.stack.last_mut() {
            scope.variables.insert(ident, link);
        } else {
            self.stack.push(ContextScope {
                variables: HashMap::from([(ident, link)]),
            });
        }
    }

    fn get_variable_err(&self, ident: &str) -> Result<Link, CompileErrorType> {
        self.get_variable(ident)
            .ok_or(CompileErrorType::UnknownName {
                name: ident.to_owned(),
            })
    }

    fn get_input(&self, name: &str) -> Result<&Rc<InputNode>, CompileErrorType> {
        if let Some(root) = &self.root {
            root.inputs.get(name).ok_or(CompileErrorType::UnknownField {
                ident: name.to_owned(),
            })
        } else {
            todo!()
            //Err(CompileErrorType::InputsAreNotAvailable)
        }
    }

    fn get_output(&self, name: &str) -> Result<&Rc<RefCell<OutputNode>>, CompileErrorType> {
        if let Some(root) = &self.root {
            root.outputs
                .get(name)
                .ok_or(CompileErrorType::UnknownField {
                    ident: name.to_owned(),
                })
        } else {
            todo!()
            //Err(CompileErrorType::OutputsAreNotAvailable)
        }
    }

    fn push_scope(&mut self) {
        self.stack.push(ContextScope::default());
    }

    fn pop_scope(&mut self) -> Option<ContextScope> {
        self.stack.pop()
    }
}

#[derive(Debug)]
pub(super) struct LogicAnalyzer<'f, 'e> {
    context: Context,
    components: Vec<Rc<Component>>,
    filename: &'f str,
    errors: &'e mut Vec<CompileError<'f>>,
}

impl<'f, 'e> LogicAnalyzer<'f, 'e> {
    pub(super) fn new(
        context: Context,
        filename: &'f str,
        errors: &'e mut Vec<CompileError<'f>>,
    ) -> Self {
        Self {
            context,
            components: Vec::new(),
            filename,
            errors,
        }
    }

    pub(super) fn into_components(self) -> Vec<Rc<Component>> {
        self.components
    }

    pub(super) fn statement(&mut self, statement: &Spanned<Statement>) {
        match &statement.inner {
            Statement::Assignment(assignment) => {
                let target = &assignment.inner.target;
                let value = &assignment.inner.value;

                if let Some(link) = self.expr_to_components(value) {
                    self.assign_link(target, link, &value.span);
                }
            }
            Statement::Let(ident, value) => {
                if let Some(link) = self.expr_to_components(value) {
                    self.context.define_variable(ident.clone(), link);
                }
            }
        }
    }

    fn expr_to_components(&mut self, expr: &Spanned<Expr>) -> Option<Link> {
        let r = match &expr.inner {
            Expr::Null => todo!(),
            Expr::BoolLiteral(_) => todo!(),
            Expr::IntLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
            ),
            Expr::FloatLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
            ),
            Expr::StringLiteral(_) => Err(CompileErrorType::StringInLogic),
            Expr::Ident(ident) => self.context.get_variable_err(ident),
            Expr::Inputs => Err(CompileErrorType::FieldAccessOnly),
            Expr::Outputs => Err(CompileErrorType::OutputsInExpression),
            Expr::Tuple(_) => todo!(),
            Expr::MemberAccess(object, field) => match &object.inner {
                Expr::Inputs => self.context.get_input(field).map(Link::node),
                _ => todo!(),
            },
            Expr::BinaryOp(op) => binary_operation(self, op)?,
            Expr::UnaryOp(op) => unary_operation(self, op)?,
            Expr::Block {
                statements,
                return_value,
            } => {
                self.context.push_scope();
                for statement in statements {
                    self.statement(statement);
                }
                let ret = return_value
                    .as_ref()
                    .and_then(|r| self.expr_to_components(r));
                self.context.pop_scope().unwrap();
                return ret;
            }
            Expr::FunctionCall { ident, props, args } => {
                self.function_call(ident, props, args, &expr.span)?
            }
        };

        match r {
            Ok(link) => Some(link),
            Err(err) => {
                self.push_error(expr.span.clone(), err);
                None
            }
        }
    }

    fn expr_to_typed_link<T>(&mut self, expr: &Spanned<Expr>) -> Option<T>
    where
        T: TryFrom<Link>,
        <T as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        if let Some(l) = self.expr_to_components(expr) {
            match T::try_from(l) {
                Ok(t) => Some(t),
                Err(err) => {
                    self.push_error(expr.span.clone(), err.into());
                    None
                }
            }
        } else {
            None
        }
    }

    fn evaluate_expr<T>(&mut self, expr: &Spanned<Expr>) -> Option<T>
    where
        T: TryFrom<EvaluatedValue<'f>>,
        <T as TryFrom<EvaluatedValue<'f>>>::Error: Into<CompileError<'f>>,
    {
        match evaluate_expr(expr, self.filename)
            .and_then(|v| T::try_from(v).map_err(|err| err.into()))
        {
            Ok(v) => Some(v),
            Err(err) => {
                self.errors.push(err);
                None
            }
        }
    }

    fn add_component(
        &mut self,
        component: Component,
        index: usize,
    ) -> Result<Link, CompileErrorType> {
        let rc = Rc::new(component);
        if let Some(link) = Link::component(&rc, index) {
            self.components.push(rc);
            Ok(link)
        } else {
            Err(CompileErrorType::NodeDoesNotExist {
                component_str: format!("{}", rc),
                index,
            })
        }
    }

    fn assign_link(&mut self, target: &Spanned<AssignmentTarget>, link: Link, value_span: &Span) {
        let r = match &target.inner {
            AssignmentTarget::Ident(_) => todo!(),
            AssignmentTarget::Inputs => todo!(),
            AssignmentTarget::Outputs => todo!(),
            AssignmentTarget::FieldAccess(object, field) => match &object.inner {
                AssignmentTarget::Ident(_) => todo!(),
                AssignmentTarget::Inputs => todo!(),
                AssignmentTarget::Outputs => self
                    .context
                    .get_output(field)
                    .map_err(|err| (&target.span, err))
                    .and_then(|o| {
                        let expected_type = o.borrow().node_type();
                        let found_type = link.node_type();
                        let success = o.borrow_mut().set_input_link(link);
                        if success {
                            Ok(())
                        } else {
                            Err((
                                value_span,
                                CompileErrorType::IncompatibleNodeType {
                                    expected_type,
                                    found_type,
                                },
                            ))
                        }
                    }),
                AssignmentTarget::FieldAccess(_, _) => todo!(),
            },
        };

        if let Err((span, err)) = r {
            self.push_error(span.clone(), err);
        }
    }

    fn push_error(&mut self, span: Span, error_type: CompileErrorType) {
        self.errors
            .push(CompileError::new(self.filename, span, error_type));
    }
}
