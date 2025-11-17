use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{ArithmeticComponent, Component, InputNode, Link, OutputNode},
    syntax::{AssignmentTarget, BinaryOp, Expr, Spanned, Statement, UnaryOp},
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
            Expr::FieldAccess(object, field) => match &object.inner {
                Expr::Inputs => self.context.get_input(field).map(Link::node),
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
                self.add_component(component, index)
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
                self.add_component(component, index)
            }
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
        };

        match r {
            Ok(link) => Some(link),
            Err(err) => {
                self.push_error(expr.span.clone(), err);
                None
            }
        }
    }

    fn expr_to_component_typed<T>(&mut self, expr: &Spanned<Expr>) -> Option<T>
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

/*#[derive(Debug)]
pub(super) enum Context<'p> {
    Root {
        inputs: Inputs,
        outputs: Outputs,
        components: Vec<Rc<Component>>,
        variables: HashMap<String, Link>,
    },
    Scoped {
        variables: HashMap<String, Link>,
        parent: &'p mut Context<'p>,
    },
}

impl<'p> Context<'p> {
    pub(super) fn new_root(inputs: Inputs, outputs: Outputs) -> Self {
        Self::Root {
            inputs,
            outputs,
            variables: HashMap::default(),
            components: Vec::new(),
        }
    }

    fn scoped(&'p mut self) -> Self {
        Self::Scoped {
            variables: HashMap::new(),
            parent: self,
        }
    }

    pub(super) fn into_components(self) -> Option<Vec<Rc<Component>>> {
        if let Self::Root { components, .. } = self {
            Some(components)
        } else {
            None
        }
    }

    pub(super) fn define_variable(&mut self, ident: String, link: Link) {
        let variables = match self {
            Self::Root { variables, .. } => variables,
            Self::Scoped { variables, .. } => variables,
        };
        variables.insert(ident, link);
    }

    pub(super) fn get_variable(&self, ident: &String) -> Result<Link, CompileErrorType> {
        let variables = match self {
            Self::Root { variables, .. } => variables,
            Self::Scoped { variables, .. } => variables,
        };
        if let Some(link) = variables.get(ident) {
            Ok(link.clone())
        } else {
            Err(CompileErrorType::UnknownName {
                name: ident.clone(),
            })
        }
    }

    fn get_input(&self, name: &str) -> Option<&Rc<InputNode>> {
        match self {
            Self::Root { inputs, .. } => inputs.get(name),
            Self::Scoped { parent, .. } => parent.get_input(name),
        }
    }

    fn get_output(&self, name: &str) -> Option<&Rc<RefCell<OutputNode>>> {
        match self {
            Self::Root { outputs, .. } => outputs.get(name),
            Self::Scoped { parent, .. } => parent.get_output(name),
        }
    }

    fn push_component(&mut self, component: Rc<Component>) {
        match self {
            Self::Root { components, .. } => components.push(component),
            Self::Scoped { parent, .. } => parent.push_component(component),
        }
    }

    pub(super) fn analyze_statements<'a>(
        &mut self,
        statements: &[Spanned<Statement>],
        filename: &'a str,
        errors: &mut Vec<CompileError<'a>>,
    ) {
        for statement in statements {
            match &statement.inner {
                Statement::Assignment(assignment) => {
                    let target = &assignment.inner.target;
                    let value = &assignment.inner.value;

                    if let Some(link) = self.expr_to_components(value, filename, errors)
                        && let Err(err) = self.assign_link(target, link, &value.span, filename)
                    {
                        errors.push(err);
                    }
                }
                Statement::Let(ident, value) => {
                    if let Some(link) = self.expr_to_components(value, filename, errors) {
                        self.define_variable(ident.clone(), link);
                    }
                }
            }
        }
    }

    fn add_component(
        &mut self,
        component: Component,
        index: usize,
    ) -> Result<Link, CompileErrorType> {
        let rc = Rc::new(component);
        let l = Link::component(&rc, index).ok_or_else(|| CompileErrorType::NodeDoesNotExist {
            component_str: format!("{}", rc),
            index,
        })?;
        self.push_component(rc);
        Ok(l)
    }

    fn expr_to_components<'a>(
        &mut self,
        expr: &Spanned<Expr>,
        filename: &'a str,
        errors: &mut Vec<CompileError<'a>>,
    ) -> Option<Link> {
        let r = match &expr.inner {
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
            Expr::Ident(ident) => self.get_variable(ident),
            Expr::Inputs => Err(CompileErrorType::FieldAccessOnly),
            Expr::Outputs => Err(CompileErrorType::OutputsInExpression),
            Expr::Tuple(_) => todo!(),
            Expr::FieldAccess(object, field) => match &object.inner {
                Expr::Inputs => self.get_input(field).map(Link::node).ok_or_else(|| {
                    CompileErrorType::UnknownField {
                        ident: field.to_owned(),
                    }
                }),
                _ => todo!(),
            },
            Expr::BinaryOp(op) => {
                let (component, index) = match op {
                    BinaryOp::Add(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Add {
                            input_a: self.expr_to_component_typed(lhs, filename, errors)?,
                            input_b: self.expr_to_component_typed(rhs, filename, errors)?,
                        }),
                        0,
                    ),
                    BinaryOp::Sub(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Subtract {
                            input_a: self.expr_to_component_typed(lhs, filename, errors)?,
                            input_b: self.expr_to_component_typed(rhs, filename, errors)?,
                        }),
                        0,
                    ),
                    BinaryOp::Mul(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Multiply {
                            input_a: self.expr_to_component_typed(lhs, filename, errors)?,
                            input_b: self.expr_to_component_typed(rhs, filename, errors)?,
                        }),
                        0,
                    ),
                    BinaryOp::Div(lhs, rhs) => (
                        Component::Arithmetic(ArithmeticComponent::Divide {
                            input_a: self.expr_to_component_typed(lhs, filename, errors)?,
                            input_b: self.expr_to_component_typed(rhs, filename, errors)?,
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
                            input_x: self.expr_to_component_typed(x, filename, errors)?,
                            function: "-x".to_owned(),
                        }),
                        0,
                    ),
                };
                self.add_component(component, index)
            }
            Expr::Block {
                statements,
                return_value,
            } => {
                let mut block = self.scoped();
                block.analyze_statements(statements, filename, errors);
                /*return_value
                .as_ref()
                .and_then(move |expr| block.expr_to_components(&expr))*/
                return None;
            }
        };

        match r {
            Ok(l) => Some(l),
            Err(err) => {
                errors.push(CompileError::new(filename, expr.span.clone(), err));
                None
            }
        }
    }

    fn expr_to_component_typed<'a, T>(
        &mut self,
        expr: &Spanned<Expr>,
        filename: &'a str,
        errors: &mut Vec<CompileError<'a>>,
    ) -> Option<T>
    where
        T: TryFrom<Link>,
        <T as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        if let Some(l) = self.expr_to_components(expr, filename, errors) {
            match T::try_from(l) {
                Ok(t) => Some(t),
                Err(err) => {
                    errors.push(CompileError::new(filename, expr.span.clone(), err.into()));
                    None
                }
            }
        } else {
            None
        }
    }

    fn assign_link<'a>(
        &mut self,
        target: &Spanned<AssignmentTarget>,
        link: Link,
        value_span: &Span,
        filename: &'a str,
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
                        let output_node = self.get_output(field);
                        if output_node.is_none() {
                            return Err(CompileError::new(
                                filename,
                                target.span.clone(),
                                CompileErrorType::UnknownField {
                                    ident: field.to_owned(),
                                },
                            ));
                        }
                        let output_node = output_node.unwrap();

                        let expected_type = output_node.borrow().node_type();
                        let found_type = link.node_type();
                        let success = output_node.borrow_mut().set_input_link(link);

                        if !success {
                            return Err(CompileError::new(
                                filename,
                                value_span.clone(),
                                CompileErrorType::IncompatibleNodeType {
                                    expected_type,
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
}*/

/*#[derive(Debug)]
pub(super) struct LogicAnalyzer<'c, 'f, 'e> {
    context: Context<'c>,
    filename: &'f str,
    errors: &'e mut Vec<CompileError<'f>>,
}

impl<'c, 'f, 'e> LogicAnalyzer<'c, 'f, 'e>
where
    'c: 'e,
    'f: 'c,
    'e: 'c,
{
    pub(super) fn new(
        inputs: Inputs,
        outputs: Outputs,
        filename: &'f str,
        errors: &'e mut Vec<CompileError<'f>>,
    ) -> Self {
        Self {
            filename,
            errors,
            context: Context::new_root(inputs, outputs),
        }
    }

    fn scoped(&'c mut self) -> Self {
        Self {
            filename: self.filename,
            errors: self.errors,
            context: Context::scoped(&mut self.context),
        }
    }

    pub(super) fn into_components(self) -> Option<Vec<Rc<Component>>> {
        self.context.into_components()
    }

    pub(super) fn statements(&mut self, statements: &[Spanned<Statement>]) {
        for statement in statements {
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
    }

    fn add_component(&mut self, component: Component, index: usize, span: &Span) -> Option<Link> {
        let rc = Rc::new(component);
        if let Some(link) = Link::component(&rc, index) {
            self.context.push_component(rc);
            Some(link)
        } else {
            self.add_error(
                span.clone(),
                CompileErrorType::NodeDoesNotExist {
                    component_str: format!("{}", rc),
                    index,
                },
            );
            None
        }
    }

    fn expr_to_components(&'c mut self, expr: &Spanned<Expr>) -> Option<Link> {
        match &expr.inner {
            Expr::BoolLiteral(_) => todo!(),
            Expr::IntLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
                &expr.span,
            ),
            Expr::FloatLiteral(v) => self.add_component(
                Component::Arithmetic(ArithmeticComponent::ConstantNumber { value: *v as f32 }),
                0,
                &expr.span,
            ),
            Expr::StringLiteral(_) => {
                self.add_error(expr.span.clone(), CompileErrorType::StringInLogic);
                None
            }
            Expr::Ident(ident) => match self.context.get_variable(ident) {
                Ok(v) => Some(v),
                Err(err) => {
                    self.add_error(expr.span.clone(), err);
                    None
                }
            },
            Expr::Inputs => {
                self.add_error(expr.span.clone(), CompileErrorType::FieldAccessOnly);
                None
            }
            Expr::Outputs => {
                self.add_error(expr.span.clone(), CompileErrorType::OutputsInExpression);
                None
            }
            Expr::Tuple(_) => todo!(),
            Expr::FieldAccess(object, field) => match &object.inner {
                Expr::Inputs => {
                    if let Some(i) = self.context.get_input(field) {
                        Some(Link::node(i))
                    } else {
                        self.add_error(
                            expr.span.clone(),
                            CompileErrorType::UnknownField {
                                ident: field.to_owned(),
                            },
                        );
                        None
                    }
                }
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
                self.add_component(component, index, &expr.span)
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
                self.add_component(component, index, &expr.span)
            }
            Expr::Block {
                statements,
                return_value,
            } => {
                let mut block = self.scoped();
                block.statements(statements);
                return_value
                    .as_ref()
                    .and_then(move |expr| block.expr_to_components(&expr))
            }
        }
    }

    fn expr_to_component_typed<T>(&mut self, expr: &Spanned<Expr>) -> Option<T>
    where
        T: TryFrom<Link>,
        <T as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        if let Some(l) = self.expr_to_components(expr) {
            match T::try_from(l) {
                Ok(t) => Some(t),
                Err(err) => {
                    self.add_error(expr.span.clone(), err.into());
                    None
                }
            }
        } else {
            None
        }
    }

    fn assign_link(&mut self, target: &Spanned<AssignmentTarget>, link: Link, value_span: &Span) {
        match &target.inner {
            AssignmentTarget::Ident(_) => todo!(),
            AssignmentTarget::Inputs => todo!(),
            AssignmentTarget::Outputs => todo!(),
            AssignmentTarget::FieldAccess(object, field) => {
                match &object.inner {
                    AssignmentTarget::Ident(_) => todo!(),
                    AssignmentTarget::Inputs => todo!(),
                    AssignmentTarget::Outputs => {
                        let output_node = self.context.get_output(field);
                        if output_node.is_none() {
                            self.add_error(
                                target.span.clone(),
                                CompileErrorType::UnknownField {
                                    ident: field.to_owned(),
                                },
                            );
                            return;
                        }
                        let output_node = output_node.unwrap();

                        let expected_type = output_node.borrow().node_type();
                        let found_type = link.node_type();
                        let success = output_node.borrow_mut().set_input_link(link);

                        if !success {
                            self.add_error(
                                value_span.clone(),
                                CompileErrorType::IncompatibleNodeType {
                                    expected_type,
                                    found_type,
                                },
                            );
                            return;
                        }
                    }
                    AssignmentTarget::FieldAccess(_, _) => todo!(),
                };
            }
        }
    }

    fn add_error(&mut self, span: Span, error_type: CompileErrorType) {
        self.errors
            .push(CompileError::new(self.filename, span, error_type));
    }
}*/
