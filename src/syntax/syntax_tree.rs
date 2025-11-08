#[expect(dead_code)]
#[derive(Debug)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: std::ops::Range<usize>,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum BinaryOp {
    Add(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Sub(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Mul(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Div(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum UnaryOp {
    Neg(Box<Spanned<Expr>>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Expr {
    Ident(String),
    LiteralValue(LiteralValue),
    FieldAccess(Box<Spanned<Expr>>, String),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Assignment {
    pub target: Spanned<Expr>,
    pub value: Spanned<Expr>,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Statement {
    Assignment(Spanned<Assignment>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct MicrocontrollerInterfaceNode {
    pub name: String,
    pub type_name: String,
    pub fields: Option<Vec<Spanned<Assignment>>>,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum MicrocontrollerInterface {
    Inputs(Vec<Spanned<MicrocontrollerInterfaceNode>>),
    Outputs(Vec<Spanned<MicrocontrollerInterfaceNode>>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum MicrocontrollerElement {
    Field(Spanned<Assignment>),
    Interface(Vec<Spanned<MicrocontrollerInterface>>),
    Logic(Vec<Spanned<Statement>>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Element {
    Microcontroller {
        name: String,
        elements: Vec<Spanned<MicrocontrollerElement>>,
    },
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct File {
    pub elements: Vec<Spanned<Element>>,
}
