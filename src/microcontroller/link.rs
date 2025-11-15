use super::{Component, InputNode};
use crate::{
    compile_error::CompileErrorType,
    microcontroller::{ComponentData, NodeType},
};

use derive_more::Deref;
use std::rc::{Rc, Weak};

#[derive(Clone, Debug)]
pub enum LinkNode {
    Node(Weak<InputNode>),
    Component(Weak<Component>, usize),
}

impl LinkNode {
    fn node(node: &Rc<InputNode>) -> Self {
        Self::Node(Rc::downgrade(node))
    }

    fn component(component: &Rc<Component>, index: usize) -> Self {
        Self::Component(Rc::downgrade(component), index)
    }
}

#[derive(Deref, Clone, Default, Debug)]
pub struct BoolLink(Option<LinkNode>);

#[derive(Deref, Clone, Default, Debug)]
pub struct NumberLink(Option<LinkNode>);

#[derive(Deref, Clone, Default, Debug)]
pub struct CompositeLink(Option<LinkNode>);

#[derive(Deref, Clone, Default, Debug)]
pub struct VideoLink(Option<LinkNode>);

#[derive(Deref, Clone, Default, Debug)]
pub struct AudioLink(Option<LinkNode>);

#[derive(Clone, Debug)]
pub enum Link {
    Bool(BoolLink),
    Number(NumberLink),
    Composite(CompositeLink),
    Video(VideoLink),
    Audio(AudioLink),
}

impl Link {
    pub fn node(node: &Rc<InputNode>) -> Self {
        let l = Some(LinkNode::node(node));
        match node.node_type() {
            NodeType::Bool => Self::Bool(BoolLink(l)),
            NodeType::Number => Self::Number(NumberLink(l)),
            NodeType::Composite => Self::Composite(CompositeLink(l)),
            NodeType::Video => Self::Video(VideoLink(l)),
            NodeType::Audio => Self::Audio(AudioLink(l)),
        }
    }

    pub fn component(component: &Rc<Component>, index: usize) -> Option<Self> {
        let l = Some(LinkNode::component(component, index));
        match component.output_type(index) {
            Some(NodeType::Bool) => Some(Self::Bool(BoolLink(l))),
            Some(NodeType::Number) => Some(Self::Number(NumberLink(l))),
            Some(NodeType::Composite) => Some(Self::Composite(CompositeLink(l))),
            Some(NodeType::Video) => Some(Self::Video(VideoLink(l))),
            Some(NodeType::Audio) => Some(Self::Audio(AudioLink(l))),
            None => None,
        }
    }

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

impl TryFrom<Link> for BoolLink {
    type Error = CompileErrorType;

    fn try_from(value: Link) -> Result<Self, Self::Error> {
        if let Link::Bool(l) = value {
            Ok(l)
        } else {
            Err(CompileErrorType::IncompatibleNodeType {
                expected_type: NodeType::Bool,
                found_type: value.node_type(),
            })
        }
    }
}

impl TryFrom<Link> for NumberLink {
    type Error = CompileErrorType;

    fn try_from(value: Link) -> Result<Self, Self::Error> {
        if let Link::Number(l) = value {
            Ok(l)
        } else {
            Err(CompileErrorType::IncompatibleNodeType {
                expected_type: NodeType::Number,
                found_type: value.node_type(),
            })
        }
    }
}

impl TryFrom<Link> for CompositeLink {
    type Error = CompileErrorType;

    fn try_from(value: Link) -> Result<Self, Self::Error> {
        if let Link::Composite(l) = value {
            Ok(l)
        } else {
            Err(CompileErrorType::IncompatibleNodeType {
                expected_type: NodeType::Composite,
                found_type: value.node_type(),
            })
        }
    }
}

impl TryFrom<Link> for VideoLink {
    type Error = CompileErrorType;

    fn try_from(value: Link) -> Result<Self, Self::Error> {
        if let Link::Video(l) = value {
            Ok(l)
        } else {
            Err(CompileErrorType::IncompatibleNodeType {
                expected_type: NodeType::Video,
                found_type: value.node_type(),
            })
        }
    }
}

impl TryFrom<Link> for AudioLink {
    type Error = CompileErrorType;

    fn try_from(value: Link) -> Result<Self, Self::Error> {
        if let Link::Audio(l) = value {
            Ok(l)
        } else {
            Err(CompileErrorType::IncompatibleNodeType {
                expected_type: NodeType::Audio,
                found_type: value.node_type(),
            })
        }
    }
}
