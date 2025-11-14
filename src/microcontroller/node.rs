use super::OptionalLink;

use derive_more::Deref;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{cell::RefCell, rc::Rc};
use strum::EnumIs;

#[derive(TryFromPrimitive, IntoPrimitive, EnumIs, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NodeMode {
    Output = 0,
    Input = 1,
}

#[derive(TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NodeType {
    Bool = 0,
    Number = 1,
    // Power = 2,
    // Fluid = 3,
    // Electric = 4,
    Composite = 5,
    Video = 6,
    Audio = 7,
    // Rope = 8,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NodePosition {
    pub x: u8,
    pub z: u8,
}

impl NodePosition {
    pub fn new(x: u8, z: u8) -> Self {
        Self { x, z }
    }
}

#[derive(Debug)]
pub struct NodeInner {
    pub label: String,
    pub description: String,
    pub node_type: NodeType,
    pub position: NodePosition,
}

#[derive(Deref, Debug)]
pub struct InputNode {
    #[deref]
    inner: NodeInner,
}

#[derive(Deref, Debug)]
pub struct OutputNode {
    #[deref]
    inner: NodeInner,
    pub input: OptionalLink,
}

#[derive(Debug)]
pub enum Node {
    Input(Rc<InputNode>),
    Output(Rc<RefCell<OutputNode>>),
}

impl Node {
    pub fn new_input(inner: NodeInner) -> Self {
        Self::Input(Rc::new(InputNode { inner }))
    }

    pub fn new_output(inner: NodeInner) -> Self {
        Self::Output(Rc::new(RefCell::new(OutputNode { inner, input: None })))
    }

    pub fn mode(&self) -> NodeMode {
        match self {
            Self::Input(_) => NodeMode::Input,
            Self::Output(_) => NodeMode::Output,
        }
    }

    pub fn label_owned(&self) -> String {
        match self {
            Self::Input(n) => n.label.to_owned(),
            Self::Output(n) => n.borrow().label.to_owned(),
        }
    }

    pub fn description_owned(&self) -> String {
        match self {
            Self::Input(n) => n.description.to_owned(),
            Self::Output(n) => n.borrow().description.to_owned(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Input(n) => n.node_type,
            Self::Output(n) => n.borrow().node_type,
        }
    }

    pub fn position(&self) -> NodePosition {
        match self {
            Self::Input(n) => n.position,
            Self::Output(n) => n.borrow().position,
        }
    }

    pub fn microcontroller_bridge_type(&self) -> Option<u8> {
        match (self.mode(), self.node_type()) {
            (NodeMode::Input, NodeType::Bool) => None,
            (NodeMode::Output, NodeType::Bool) => Some(1),
            (NodeMode::Input, NodeType::Number) => Some(2),
            (NodeMode::Output, NodeType::Number) => Some(3),
            (NodeMode::Input, NodeType::Composite) => Some(4),
            (NodeMode::Output, NodeType::Composite) => Some(5),
            (NodeMode::Input, NodeType::Video) => Some(6),
            (NodeMode::Output, NodeType::Video) => Some(7),
            (NodeMode::Input, NodeType::Audio) => Some(8),
            (NodeMode::Output, NodeType::Audio) => Some(9),
        }
    }
}
