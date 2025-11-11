use super::{
    Component, OutputNode, PositionedComponent, PositionedMicrocontroller, PositionedNode,
    UnpositionedMicrocontroller,
};
use crate::microcontroller::{ComponentPosition, InputNode, Link, Microcontroller, Node};

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::{Rc, Weak},
};

// Rc を HashMap のキーにするために usize に変換
fn rc_key<T>(v: &Rc<T>) -> usize {
    Rc::as_ptr(v) as usize
}

fn weak_key<T>(v: &Weak<T>) -> usize {
    v.as_ptr() as usize
}

// コンポーネントの識別子
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum ComponentKey {
    InputNode(usize),
    OutputNode(usize),
    Component(usize),
}

impl From<&Rc<InputNode>> for ComponentKey {
    fn from(value: &Rc<InputNode>) -> Self {
        Self::InputNode(rc_key(value))
    }
}

impl From<&Rc<OutputNode>> for ComponentKey {
    fn from(value: &Rc<OutputNode>) -> Self {
        Self::OutputNode(rc_key(value))
    }
}

impl From<&Rc<Component>> for ComponentKey {
    fn from(value: &Rc<Component>) -> Self {
        Self::Component(rc_key(value))
    }
}

impl From<&Weak<InputNode>> for ComponentKey {
    fn from(value: &Weak<InputNode>) -> Self {
        Self::InputNode(weak_key(value))
    }
}

impl From<&Weak<OutputNode>> for ComponentKey {
    fn from(value: &Weak<OutputNode>) -> Self {
        Self::OutputNode(weak_key(value))
    }
}

impl From<&Weak<Component>> for ComponentKey {
    fn from(value: &Weak<Component>) -> Self {
        Self::Component(weak_key(value))
    }
}

impl From<&Link> for ComponentKey {
    fn from(value: &Link) -> Self {
        match value {
            Link::Node(n) => n.into(),
            Link::Component(c, _) => c.into(),
        }
    }
}

// 各コンポーネントの左右の接続
#[derive(Default, Debug)]
struct GraphConnectionItem {
    left: HashSet<ComponentKey>,
    right: HashSet<ComponentKey>,
}

impl GraphConnectionItem {
    fn to_left(left: ComponentKey) -> Self {
        Self {
            left: HashSet::from([left]),
            right: HashSet::new(),
        }
    }

    fn to_right(right: ComponentKey) -> Self {
        Self {
            left: HashSet::new(),
            right: HashSet::from([right]),
        }
    }
}

// 全体の接続関係
#[derive(Default, Debug)]
struct GraphConnection {
    inner: HashMap<ComponentKey, GraphConnectionItem>,
}

impl GraphConnection {
    fn new(nodes: &[Node], components: &[Rc<Component>]) -> Self {
        let mut s = Self::with_capacity(nodes.len() + components.len());

        for node in nodes {
            if let Node::Output(n) = node
                && let Some(link) = &n.input
            {
                s.link_left(n.into(), link);
            }
        }

        for component in components {
            for link in component.input_links().into_iter().flatten() {
                s.link_left(component.into(), link);
            }
        }

        s
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    fn connect(&mut self, left: ComponentKey, right: ComponentKey) {
        if let Some(s) = self.inner.get_mut(&right) {
            s.left.insert(left);
        } else {
            self.inner.insert(right, GraphConnectionItem::to_left(left));
        }

        if let Some(s) = self.inner.get_mut(&left) {
            s.right.insert(right);
        } else {
            self.inner
                .insert(left, GraphConnectionItem::to_right(right));
        }
    }

    fn link_left(&mut self, k: ComponentKey, link: &Link) {
        self.connect(link.into(), k);
    }

    fn make_islands(&self) -> Vec<Island> {
        let mut visited = HashSet::new();
        let mut islands = Vec::new();

        for k in self.inner.keys() {
            if visited.contains(k) {
                continue;
            }
            let island = construct_island(self, *k);
            for v in &island.all {
                visited.insert(*v);
            }
            islands.push(island);
        }

        islands
    }
}

// 連結している島
#[derive(Default, Debug)]
struct Island {
    inner: BTreeMap<i32, HashSet<ComponentKey>>,
    all: HashSet<ComponentKey>,
}

impl Island {
    fn insert(&mut self, k: i32, value: ComponentKey) {
        if let Some(s) = self.inner.get_mut(&k) {
            s.insert(value);
        } else {
            self.inner.insert(k, HashSet::from([value]));
        }
        self.all.insert(value);
    }

    fn contains(&self, value: &ComponentKey) -> bool {
        self.all.contains(value)
    }
}

fn construct_island(connection: &GraphConnection, origin: ComponentKey) -> Island {
    // item を起点に接続を辿って島を形成する
    let mut island = Island::default();
    island.insert(0, origin);

    let c = connection.inner.get(&origin).unwrap();
    let mut stack = DualStack::new(
        c.right.iter().map(|v| (1, *v)).collect::<Vec<_>>(),
        c.left.iter().map(|v| (-1, *v)).collect::<Vec<_>>(),
    );
    loop {
        let p = stack.pop();
        if p.is_none() {
            break;
        }
        let (x, node) = p.unwrap();
        if island.contains(&node) {
            continue;
        }

        island.insert(x, node);
        let c = connection.inner.get(&node).unwrap();
        for v in &c.right {
            if !island.contains(v) {
                stack.push_primary((x + 1, *v));
            }
        }
        for v in &c.left {
            if !island.contains(v) {
                stack.push_secondary((x - 1, *v));
            }
        }
    }

    island
}

// primary を消費しきってから secondary に行くスタック
#[derive(Default, Debug)]
struct DualStack<T> {
    primary: Vec<T>,
    secondary: Vec<T>,
}

impl<T> DualStack<T> {
    fn new(primary: Vec<T>, secondary: Vec<T>) -> Self {
        Self { primary, secondary }
    }

    fn push_primary(&mut self, value: T) {
        self.primary.push(value);
    }

    fn push_secondary(&mut self, value: T) {
        self.secondary.push(value);
    }

    fn pop(&mut self) -> Option<T> {
        if !self.primary.is_empty() {
            self.primary.pop()
        } else {
            self.secondary.pop()
        }
    }
}

fn auto_layout(nodes: &[Node], components: &[Rc<Component>]) {
    // キーから実体を参照できるように
    /*let mut input_node_map = HashMap::new();
    let mut output_node_map = HashMap::new();
    for node in nodes {
        match node {
            Node::Input(n) => {
                input_node_map.insert(ComponentKey::from(n), n.clone());
            }
            Node::Output(n) => {
                output_node_map.insert(ComponentKey::from(n), n.clone());
            }
        }
    }
    let component_map = components
        .iter()
        .map(|c| (ComponentKey::from(c), c.clone()))
        .collect::<HashMap<_, _>>();*/

    let connection = GraphConnection::new(nodes, components);
    let islands = connection.make_islands();

    dbg!(&connection);
    dbg!(islands);
}

#[derive(Debug)]
struct AutoLayout {
    microcontroller: UnpositionedMicrocontroller,
}

impl AutoLayout {
    fn new(microcontroller: UnpositionedMicrocontroller) -> Self {
        Self { microcontroller }
    }

    fn layout(&mut self) {
        auto_layout(
            &self.microcontroller.nodes,
            &self.microcontroller.components,
        );
    }
}

impl UnpositionedMicrocontroller {
    pub fn auto_layout(self) -> PositionedMicrocontroller {
        /*let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in self.nodes {
            nodes.push(PositionedNode {
                inner: node,
                component_position: ComponentPosition::new(0, 0),
            });
        }

        let mut components = Vec::with_capacity(self.components.len());
        for component in self.components {
            components.push(PositionedComponent {
                inner: component,
                position: ComponentPosition::new(0, 0),
            });
        }*/

        let mut layouter = AutoLayout::new(self);
        layouter.layout();
        let mc = layouter.microcontroller;

        Microcontroller {
            name: mc.name,
            description: mc.description,
            width: mc.width,
            length: mc.length,
            nodes: Vec::new(),
            components: Vec::new(),
        }
    }
}
