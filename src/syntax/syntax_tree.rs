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
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum UnaryOp {
    Neg(Box<Expr>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Expr {
    Ident(String),
    LiteralValue(LiteralValue),
    FieldAccess { target: Box<Expr>, field: String },
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Assignment {
    pub target: Expr,
    pub value: Expr,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Statement {
    Assignment(Assignment),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct MicrocontrollerInterfaceNode {
    pub name: String,
    pub type_name: String,
    pub fields: Option<Vec<Assignment>>,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum MicrocontrollerInterface {
    Inputs(Vec<MicrocontrollerInterfaceNode>),
    Outputs(Vec<MicrocontrollerInterfaceNode>),
    Properties,
    Tooltips,
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum MicrocontrollerElement {
    Field(Assignment),
    Interface(Vec<MicrocontrollerInterface>),
    Logic(Vec<Statement>),
}

#[expect(dead_code)]
#[derive(Debug)]
pub enum Element {
    Microcontroller {
        name: String,
        elements: Vec<MicrocontrollerElement>,
    },
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct File {
    pub elements: Vec<Element>,
}
