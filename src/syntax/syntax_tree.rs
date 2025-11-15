use derive_more::Deref;

#[derive(Deref, Debug)]
pub struct Spanned<T> {
    #[deref]
    pub inner: T,
    pub span: std::ops::Range<usize>,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Sub(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Mul(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Div(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg(Box<Spanned<Expr>>),
}

#[derive(Debug)]
pub enum Expr {
    BoolLiteral(bool),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    Ident(String),
    Inputs,
    Outputs,
    Tuple(Vec<Spanned<Expr>>),
    FieldAccess(Box<Spanned<Expr>>, String),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Block {
        statements: Vec<Spanned<Statement>>,
        return_value: Option<Box<Spanned<Expr>>>,
    },
}

#[derive(Debug)]
pub enum AssignmentTarget {
    Ident(String),
    Inputs,
    Outputs,
    FieldAccess(Box<Spanned<AssignmentTarget>>, String),
}

#[derive(Debug)]
pub struct Assignment {
    pub target: Spanned<AssignmentTarget>,
    pub value: Spanned<Expr>,
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Spanned<Expr>),
    Assignment(Spanned<Assignment>),
}

#[derive(Debug)]
pub struct MicrocontrollerInterfaceNode {
    pub name: String,
    pub type_name: String,
    pub fields: Option<Vec<Spanned<Assignment>>>,
}

#[derive(Debug)]
pub enum MicrocontrollerInterface {
    Inputs(Vec<Spanned<MicrocontrollerInterfaceNode>>),
    Outputs(Vec<Spanned<MicrocontrollerInterfaceNode>>),
}

#[derive(Debug)]
pub enum MicrocontrollerElement {
    Field(Spanned<Assignment>),
    Interface(Vec<Spanned<MicrocontrollerInterface>>),
    Logic(Vec<Spanned<Statement>>),
}

#[derive(Debug)]
pub enum Element {
    Microcontroller {
        name: String,
        elements: Vec<Spanned<MicrocontrollerElement>>,
    },
}

#[derive(Debug)]
pub struct File {
    pub elements: Vec<Spanned<Element>>,
}
