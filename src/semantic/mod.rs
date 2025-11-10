use std::collections::HashMap;

use crate::{
    microcontroller::{Microcontroller, UnpositionedMicrocontroller},
    syntax::{self, Spanned},
};

pub fn analyze_file(
    tree: &Spanned<syntax::File>,
) -> Result<HashMap<String, UnpositionedMicrocontroller>, ()> {
    let mut microcontrollers = HashMap::new();
    for element in &tree.elements {
        match &element.inner {
            syntax::Element::Microcontroller { name, elements } => {
                microcontrollers.insert(name.clone(), analyze_microcontroller(elements)?);
            }
        }
    }

    Ok(microcontrollers)
}

pub fn analyze_microcontroller(
    elements: &[Spanned<syntax::MicrocontrollerElement>],
) -> Result<UnpositionedMicrocontroller, ()> {
    let mut name = String::new();
    let mut description = String::new();
    let mut width = 0;
    let mut height = 0;

    for element in elements {
        match &element.inner {
            syntax::MicrocontrollerElement::Field(assignment) => {}
            syntax::MicrocontrollerElement::Interface(interfaces) => {}
            syntax::MicrocontrollerElement::Logic(statements) => {}
        }
    }

    Ok(Microcontroller {
        name: todo!(),
        description: todo!(),
        width: todo!(),
        length: todo!(),
        nodes: todo!(),
        components: todo!(),
    })
}
