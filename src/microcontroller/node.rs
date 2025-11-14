use super::{AudioLink, BoolLink, CompositeLink, NumberLink, VideoLink};
use crate::microcontroller::{Link, LinkNode};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{cell::RefCell, ops::Deref, rc::Rc};
use strum::EnumIs;

#[derive(TryFromPrimitive, IntoPrimitive, EnumIs, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NodeMode {
    Output = 0,
    Input = 1,
}

#[derive(TryFromPrimitive, IntoPrimitive, strum::Display, PartialEq, Eq, Clone, Copy, Debug)]
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
    //pub node_type: NodeType,
    pub position: NodePosition,
}

#[derive(Debug)]
pub enum InputNode {
    Bool(NodeInner),
    Number(NodeInner),
    Composite(NodeInner),
    Video(NodeInner),
    Audio(NodeInner),
}

impl InputNode {
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Bool(_) => NodeType::Bool,
            Self::Number(_) => NodeType::Number,
            Self::Composite(_) => NodeType::Composite,
            Self::Video(_) => NodeType::Video,
            Self::Audio(_) => NodeType::Audio,
        }
    }
}

impl Deref for InputNode {
    type Target = NodeInner;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Bool(inner) => inner,
            Self::Number(inner) => inner,
            Self::Composite(inner) => inner,
            Self::Video(inner) => inner,
            Self::Audio(inner) => inner,
        }
    }
}

#[derive(Debug)]
pub enum OutputNode {
    Bool {
        inner: NodeInner,
        input: BoolLink,
    },
    Number {
        inner: NodeInner,
        input: NumberLink,
    },
    Composite {
        inner: NodeInner,
        input: CompositeLink,
    },
    Video {
        inner: NodeInner,
        input: VideoLink,
    },
    Audio {
        inner: NodeInner,
        input: AudioLink,
    },
}

impl OutputNode {
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Bool { .. } => NodeType::Bool,
            Self::Number { .. } => NodeType::Number,
            Self::Composite { .. } => NodeType::Composite,
            Self::Video { .. } => NodeType::Video,
            Self::Audio { .. } => NodeType::Audio,
        }
    }

    pub fn input_link_node(&self) -> &Option<LinkNode> {
        match self {
            Self::Bool { input, .. } => input,
            Self::Number { input, .. } => input,
            Self::Composite { input, .. } => input,
            Self::Video { input, .. } => input,
            Self::Audio { input, .. } => input,
        }
    }

    pub fn set_input_link(&mut self, link: Link) -> bool {
        match (self, link) {
            (Self::Bool { input, .. }, Link::Bool(l)) => *input = l,
            (Self::Number { input, .. }, Link::Number(l)) => *input = l,
            (Self::Composite { input, .. }, Link::Composite(l)) => *input = l,
            (Self::Video { input, .. }, Link::Video(l)) => *input = l,
            (Self::Audio { input, .. }, Link::Audio(l)) => *input = l,
            _ => {
                return false;
            }
        }
        true
    }
}

impl Deref for OutputNode {
    type Target = NodeInner;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Bool { inner, .. } => inner,
            Self::Number { inner, .. } => inner,
            Self::Composite { inner, .. } => inner,
            Self::Video { inner, .. } => inner,
            Self::Audio { inner, .. } => inner,
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Input(Rc<InputNode>),
    Output(Rc<RefCell<OutputNode>>),
}

impl Node {
    pub fn new_input(inner: NodeInner, node_type: NodeType) -> Self {
        let n = match node_type {
            NodeType::Bool => InputNode::Bool(inner),
            NodeType::Number => InputNode::Number(inner),
            NodeType::Composite => InputNode::Composite(inner),
            NodeType::Video => InputNode::Video(inner),
            NodeType::Audio => InputNode::Audio(inner),
        };
        Self::Input(Rc::new(n))
    }

    pub fn new_output(inner: NodeInner, node_type: NodeType) -> Self {
        let n = match node_type {
            NodeType::Bool => OutputNode::Bool {
                inner,
                input: Default::default(),
            },
            NodeType::Number => OutputNode::Number {
                inner,
                input: Default::default(),
            },
            NodeType::Composite => OutputNode::Composite {
                inner,
                input: Default::default(),
            },
            NodeType::Video => OutputNode::Video {
                inner,
                input: Default::default(),
            },
            NodeType::Audio => OutputNode::Audio {
                inner,
                input: Default::default(),
            },
        };
        Self::Output(Rc::new(RefCell::new(n)))
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
            Self::Input(n) => n.node_type(),
            Self::Output(n) => n.borrow().node_type(),
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
