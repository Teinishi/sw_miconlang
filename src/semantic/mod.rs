mod evaluate_expr;
mod field_analyzer;
mod interface;
//mod logic;
mod logic_analyzer;
mod value_type;
use evaluate_expr::evaluate_expr;
use field_analyzer::FieldAnalyzer;
use interface::InterfaceAnalyzer;
use logic_analyzer::{Context, LogicAnalyzer};
pub use value_type::ValueType;

use crate::{
    compile_error::CompileError,
    microcontroller::{Component, Node, UnpositionedMicrocontroller},
    syntax::{self, MicrocontrollerElement, Spanned},
};

use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct FileAnalyzeResult<'a> {
    microcontrollers: HashMap<String, UnpositionedMicrocontroller>,
    errors: Vec<CompileError<'a>>,
}

impl<'a> FileAnalyzeResult<'a> {
    pub fn into_output(self) -> Option<HashMap<String, UnpositionedMicrocontroller>> {
        if self.has_errors() {
            None
        } else {
            Some(self.microcontrollers)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<CompileError<'a>> {
        &self.errors
    }
}

pub fn analyze_file<'a>(tree: &Spanned<syntax::File>, filename: &'a str) -> FileAnalyzeResult<'a> {
    let mut microcontrollers = HashMap::new();
    let mut errors = Vec::new();

    for element in &tree.elements {
        match &element.inner {
            syntax::Element::Microcontroller { name, elements } => {
                if let Some(mc) = analyze_microcontroller(elements, filename, &mut errors) {
                    microcontrollers.insert(name.clone(), mc);
                }
            }
        }
    }

    FileAnalyzeResult {
        microcontrollers,
        errors,
    }
}

#[derive(Default, Debug)]
struct MicrocontrollerField {
    name: Option<String>,
    description: Option<String>,
    size: Option<(u8, u8)>,
}

impl MicrocontrollerField {
    fn into_microcontroller(
        self,
        nodes: Vec<Node>,
        components: Vec<Rc<Component>>,
    ) -> UnpositionedMicrocontroller {
        let (width, length) = self.size.unwrap_or((1, 1));
        UnpositionedMicrocontroller {
            name: self.name.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            width,
            length,
            nodes,
            components,
        }
    }
}

fn analyze_microcontroller<'a>(
    elements: &[Spanned<syntax::MicrocontrollerElement>],
    filename: &'a str,
    errors: &mut Vec<CompileError<'a>>,
) -> Option<UnpositionedMicrocontroller> {
    let mut mc = MicrocontrollerField::default();

    let mut fields = FieldAnalyzer::new(filename);
    for element in elements {
        if let MicrocontrollerElement::Field(assignment) = &element.inner {
            let r = fields.assignment(assignment, |ident, expr| {
                match ident.as_str() {
                    "name" => mc.name = Some(evaluate_expr(expr, filename)?.try_into()?),
                    "description" => {
                        mc.description = Some(evaluate_expr(expr, filename)?.try_into()?)
                    }
                    "size" => {
                        mc.size = Some(
                            evaluate_expr(expr, filename)?
                                .tuple_int_ranged(vec![1..=6, 1..=6])?
                                .try_into()?,
                        )
                    }
                    _ => {
                        return Ok(false);
                    }
                }
                Ok(true)
            });
            if let Err(err) = r {
                errors.push(err);
            }
        }
    }
    if !errors.is_empty() {
        return None;
    }

    let mut interface = InterfaceAnalyzer::new(filename, mc.size);
    for element in elements {
        if let MicrocontrollerElement::Interface(items) = &element.inner {
            for item in items {
                interface.element(item, errors);
            }
        }
    }
    if !errors.is_empty() {
        return None;
    }
    let interface = interface.layout();
    mc.size = Some(interface.size);

    let mut logic_analyzer = LogicAnalyzer::new(
        Context::new(interface.inputs, interface.outputs),
        filename,
        errors,
    );
    for element in elements {
        if let MicrocontrollerElement::Logic(statements) = &element.inner {
            for statement in statements {
                logic_analyzer.statement(statement);
            }
        }
    }
    let components = logic_analyzer.into_components();
    if !errors.is_empty() {
        return None;
    }

    Some(mc.into_microcontroller(interface.nodes, components))
}
