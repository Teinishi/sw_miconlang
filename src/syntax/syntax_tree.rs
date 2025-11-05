#[derive(Debug)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub struct Expr {
    pub value: LiteralValue,
}

#[derive(Debug)]
pub struct File {
    pub elements: Vec<Element>,
}

#[derive(Debug)]
pub enum Element {
    Microcontroller {
        name: String,
        fields: Vec<(String, Expr)>,
    },
}
