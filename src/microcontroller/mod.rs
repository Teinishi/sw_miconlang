mod auto_layout;
mod component;
mod link;
mod node;

pub use component::Component;
pub use link::{Link, OptionalLink};
#[expect(unused_imports)]
pub use node::{InputNode, Node, NodeMode, NodePosition, NodeType, OutputNode};

use derive_more::Deref;
use std::rc::Rc;

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

#[derive(Deref, Debug)]
pub struct PositionedComponent {
    #[deref]
    pub inner: Rc<Component>,
    pub position: ComponentPosition,
}
