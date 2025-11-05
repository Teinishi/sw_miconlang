#[expect(dead_code)]
#[derive(Debug)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Expr {
    pub value: LiteralValue,
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Assignment {
    pub ident: String,
    pub value: Expr,
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
