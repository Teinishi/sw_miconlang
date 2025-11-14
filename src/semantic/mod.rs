mod evaluate_expr;
mod field_analyzer;
mod interface;
mod value_type;
use evaluate_expr::evaluate_expr;
use field_analyzer::FieldAnalyzer;
use interface::analyze_microcontroller_interfaces;
pub use value_type::ValueType;

use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{Component, Node, UnpositionedMicrocontroller},
    syntax::{self, Spanned},
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
struct PartialMicrocontroller {
    name: Option<String>,
    description: Option<String>,
    size: Option<(u8, u8)>,
    nodes: Option<Vec<Node>>,
    components: Option<Vec<Rc<Component>>>,
}

impl From<PartialMicrocontroller> for UnpositionedMicrocontroller {
    fn from(value: PartialMicrocontroller) -> Self {
        let (width, length) = value.size.unwrap_or((1, 1));
        Self {
            name: value.name.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
            width,
            length,
            nodes: value.nodes.unwrap_or_default(),
            components: value.components.unwrap_or_default(),
        }
    }
}

fn analyze_microcontroller<'a>(
    elements: &[Spanned<syntax::MicrocontrollerElement>],
    filename: &'a str,
    errors: &mut Vec<CompileError<'a>>,
) -> Option<UnpositionedMicrocontroller> {
    let mut mc = PartialMicrocontroller::default();
    let mut fields = FieldAnalyzer::new(filename);
    let mut interface: Option<interface::Interface> = None;

    for element in elements {
        match &element.inner {
            syntax::MicrocontrollerElement::Field(assignment) => {
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
            syntax::MicrocontrollerElement::Interface(interfaces) => {
                if interface.is_none() {
                    interface = Some(analyze_microcontroller_interfaces(
                        interfaces, &mut mc, filename, errors,
                    ));
                } else {
                    errors.push(CompileError::new(
                        filename,
                        element.span.clone(),
                        CompileErrorType::ElementAlreadyDeclared,
                    ));
                }
            }
            _ => {}
        }
    }

    let mut logic = false;
    for element in elements {
        if let syntax::MicrocontrollerElement::Logic(statements) = &element.inner {
            if !logic {
                logic = true;
                dbg!(statements);
            } else {
                errors.push(CompileError::new(
                    filename,
                    element.span.clone(),
                    CompileErrorType::ElementAlreadyDeclared,
                ));
            }
        }
    }

    Some(mc.into())
}
