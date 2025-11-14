mod arithmetic;
pub use arithmetic::ArithmeticComponent;

use super::{LinkNode, NodeType, NumberLink};
use crate::xml_schema::{ObjectValue, ObjectValueTag};

use enum_dispatch::enum_dispatch;
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[enum_dispatch]
pub trait ComponentData {
    fn component_type(&self) -> u8;
    fn height(&self) -> u8;
    fn input_links_node(&self) -> Vec<&Option<LinkNode>>;
    fn attrs(&self) -> Option<HashMap<String, String>>;
    fn value_list(&self) -> Option<Vec<(ObjectValueTag, ObjectValue)>>;
    fn output_type(&self, index: usize) -> Option<NodeType>;
    //fn inputs(&self) -> Cow<'static, [ComponentNode<'static>]>;
    //fn outputs(&self) -> Cow<'static, [ComponentNode<'static>]>;
}

#[derive(Debug)]
#[enum_dispatch(ComponentData)]
pub enum Component {
    Arithmetic(ArithmeticComponent),
}

impl Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arithmetic(c) => Display::fmt(c, f),
        }
    }
}

fn single_attr(name: &str, value: String) -> HashMap<String, String> {
    if value.is_empty() {
        HashMap::new()
    } else {
        HashMap::from([(name.to_owned(), value)])
    }
}

#[expect(dead_code)]
#[derive(Clone, Debug)]
pub struct ComponentNode<'a>(Cow<'a, str>, NodeType);
