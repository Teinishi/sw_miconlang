mod field;
mod interface;
mod value_type;
use crate::{
    compile_error::CompileError,
    microcontroller::{Microcontroller, UnpositionedMicrocontroller},
    syntax::{self, Assignment, Spanned},
};
use field::{MutField, analyze_field};
use interface::analyze_microcontroller_interfaces;
pub use value_type::ValueType;

use std::collections::HashMap;

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

fn analyze_microcontroller<'a>(
    elements: &[Spanned<syntax::MicrocontrollerElement>],
    filename: &'a str,
    errors: &mut Vec<CompileError<'a>>,
) -> Option<UnpositionedMicrocontroller> {
    let mut mc = Microcontroller::default();

    for element in elements {
        match &element.inner {
            syntax::MicrocontrollerElement::Field(assignment) => {
                if let Err(err) = analyze_microcontroller_field(assignment, &mut mc, filename) {
                    errors.push(err);
                }
            }
            syntax::MicrocontrollerElement::Interface(interfaces) => {
                analyze_microcontroller_interfaces(interfaces, &mut mc, filename, errors);
            }
            syntax::MicrocontrollerElement::Logic(statements) => {
                dbg!(&statements);
            }
        }
    }

    Some(mc)
}

fn analyze_microcontroller_field<'a>(
    assignment: &Spanned<Assignment>,
    mc: &mut UnpositionedMicrocontroller,
    filename: &'a str,
) -> Result<(), CompileError<'a>> {
    let mut name = None;
    let mut description = None;
    let mut width = None;
    let mut length = None;

    analyze_field(assignment, filename, |ident| match ident.as_str() {
        "name" => Some(MutField::String(&mut name)),
        "description" => Some(MutField::String(&mut description)),
        "width" => Some(MutField::RangedU8(&mut width, 1..=6)),
        "length" => Some(MutField::RangedU8(&mut length, 1..=6)),
        _ => None,
    })?;

    if let Some(name) = name {
        mc.name = name;
    }
    if let Some(description) = description {
        mc.description = description;
    }
    if let Some(width) = width {
        mc.width = width;
    }
    if let Some(length) = length {
        mc.length = length;
    }

    Ok(())
}
