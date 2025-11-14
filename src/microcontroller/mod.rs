mod auto_layout;
mod component;
mod link;
mod node;

pub use component::Component;
pub use link::{Link, OptionalLink};
pub use node::{InputNode, Node, NodeInner, NodeMode, NodePosition, NodeType, OutputNode};

use crate::xml_schema::{self, ObjectValue, ObjectValueTag};

use derive_more::Deref;
use std::{collections::BTreeMap, rc::Rc};

pub type UnpositionedMicrocontroller = Microcontroller<Node, Rc<Component>>;
pub type PositionedMicrocontroller = Microcontroller<PositionedNode, PositionedComponent>;

#[derive(Debug)]
pub struct Microcontroller<N, C> {
    pub name: String,
    pub description: String,
    pub width: u8,
    pub length: u8,
    pub nodes: Vec<N>,
    pub components: Vec<C>,
}

impl<N, C> Default for Microcontroller<N, C> {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            width: 1,
            length: 1,
            nodes: Vec::new(),
            components: Vec::new(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ComponentPosition {
    pub x: i32,
    pub y: i32,
}

impl ComponentPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Deref, Debug)]
pub struct PositionedNode {
    #[deref]
    pub inner: Node,
    pub component_position: ComponentPosition,
}

impl PositionedNode {
    pub fn as_xml_item(&self, id: u32) -> xml_schema::ComponentItem {
        xml_schema::ComponentItem {
            component_type: self.microcontroller_bridge_type(),
            object: xml_schema::ComponentObject {
                id,
                pos: option_component_pos(self.component_position.clone()),
                in_map: BTreeMap::new(),
                ..Default::default()
            },
        }
    }
}

#[derive(Deref, Debug)]
pub struct PositionedComponent {
    #[deref]
    pub inner: Rc<Component>,
    pub position: ComponentPosition,
}

impl PositionedComponent {
    pub fn as_xml_item(&self, id: u32) -> xml_schema::ComponentItem {
        let value_list = match &*self.inner {
            Component::ConstantNumber { value } => {
                vec![(ObjectValueTag::N, object_value(*value))]
            }
            _ => vec![],
        };

        xml_schema::ComponentItem {
            component_type: option_u8(self.component_type()),
            object: xml_schema::ComponentObject {
                id,
                pos: option_component_pos(self.position.clone()),
                value_list,
                ..Default::default()
            },
        }
    }
}

fn object_value(value: f32) -> ObjectValue {
    ObjectValue {
        text: Some(value.to_string()),
        value: option_f32(value).map(|v| f32::to_string(&v)),
    }
}

fn to_option<T: PartialEq>(value: T, default: T) -> Option<T> {
    if value == default { None } else { Some(value) }
}

fn option_u8(value: u8) -> Option<u8> {
    to_option(value, 0)
}

fn option_f32(value: f32) -> Option<f32> {
    to_option(value, 0.0)
}

fn option_component_pos(value: ComponentPosition) -> Option<xml_schema::ComponentPos> {
    if value.x == 0 && value.y == 0 {
        None
    } else {
        Some(xml_schema::ComponentPos {
            x: option_f32(0.25 * (value.x as f32)),
            y: option_f32(0.25 * (value.y as f32)),
        })
    }
}
