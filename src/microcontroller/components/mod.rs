mod arithmetic;
pub use arithmetic::ArithmeticComponent;

use super::{NodeType, OptionalLink};
use crate::xml_schema::{ObjectValue, ObjectValueTag};

use enum_dispatch::enum_dispatch;
use std::{borrow::Cow, collections::HashMap};

#[expect(dead_code)]
#[enum_dispatch]
pub trait ComponentData {
    fn component_type(&self) -> u8;
    fn height(&self) -> u8;
    fn input_links(&self) -> Vec<&OptionalLink>;
    fn attrs(&self) -> Option<HashMap<String, String>>;
    fn value_list(&self) -> Option<Vec<(ObjectValueTag, ObjectValue)>>;
    fn inputs(&self) -> Cow<'static, [ComponentNode<'static>]>;
    fn outputs(&self) -> Cow<'static, [ComponentNode<'static>]>;
}

#[derive(Debug)]
#[enum_dispatch(ComponentData)]
pub enum Component {
    Arithmetic(ArithmeticComponent),
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
