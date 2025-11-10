use super::{Component, InputNode};

use std::rc::{Rc, Weak};

pub type OptionalLink = Option<Link>;

#[derive(Debug)]
pub enum Link {
    Node(Weak<InputNode>),
    Component(Weak<Component>, usize),
}

impl Link {
    pub fn node(node: &Rc<InputNode>) -> Self {
        Self::Node(Rc::downgrade(node))
    }

    pub fn component(component: &Rc<Component>, index: usize) -> Self {
        Self::Component(Rc::downgrade(component), index)
    }
}
