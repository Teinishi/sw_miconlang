use super::{
    Component, OutputNode, PositionedComponent, PositionedMicrocontroller, PositionedNode,
    UnpositionedMicrocontroller,
};
use crate::microcontroller::{ComponentPosition, InputNode, Link, Microcontroller, Node};

use std::rc::{Rc, Weak};

// Rc を HashMap のキーにするために usize に変換
fn rc_key<T>(v: &Rc<T>) -> usize {
    Rc::as_ptr(v) as usize
}

fn weak_key<T>(v: &Weak<T>) -> usize {
    v.as_ptr() as usize
}

#[derive(Debug)]
struct Island {
    x: i32,
    self_height: i32,
    height: i32,
    next: Option<Vec<(i32, Island)>>,
}

impl Island {
    fn new(x: i32, self_height: i32, links: Option<&[Link]>) -> Self {
        todo!();

        let height = next
            .as_ref()
            .map(|v| v.iter().map(|(_, n)| n.height).sum())
            .unwrap_or(0)
            .max(self_height);

        Self {
            x,
            self_height,
            height,
            next,
        }
    }

    fn input_node(_node: &Rc<InputNode>, x: i32) -> Self {
        Self::new(x, 2, None)
    }

    fn output_node(node: &Rc<OutputNode>, x: i32) -> Self {
        /*let next = match &node.input {
            Some(Link::Node(n)) => n.upgrade().map(|n| [Self::input_node(&n, x + 1)]),
            Some(Link::Component(c, i)) => c.upgrade().map(|c| [Self::component(&c, *i, x + 1)]),
            None => None,
        };*/

        Self::new(x, 2, node.input.as_ref().map(|s| std::slice::from_ref(s)))
    }

    fn component(component: &Rc<Component>, node_index: usize, x: i32) -> Self {
        /*let next_islands: Vec<Island> = component
            .input_links()
            .iter()
            .filter_map(|l| match l {
                Some(Link::Node(n)) => n.upgrade().map(|n| Self::input_node(&n, x + 1)),
                Some(Link::Component(c, i)) => c.upgrade().map(|c| Self::component(&c, *i, x + 1)),
                None => None,
            })
            .collect();

        let self_height = component.height() as i32;
        let total_height: i32 = next_islands.iter().map(|n| n.height).sum();
        let mut y = self_height + (total_height - self_height) / 2;

        let mut next = Vec::with_capacity(next_islands.len());
        for n in next_islands {
            let h = n.height;
            next.push((y - (h - n.self_height) / 2 - n.self_height, n));
            y -= h;
        }

        Self::new(
            x,
            self_height,
            if next_islands.is_empty() {
                None
            } else {
                Some(next)
            },
        )*/

        todo!()
    }
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
        let output_nodes = self.microcontroller.nodes.iter().filter_map(|n| match n {
            Node::Output(node) => Some(node),
            _ => None,
        });

        for output_node in output_nodes {
            // 出力ノードから遡って位置決め
            Island::output_node(output_node, 0);

            /*let x = 0;
            let y = 0;
            let position = ComponentPosition::new(x, y);
            self.node_position_map.insert(rc_key(output_node), position);

            match &output_node.input {
                Some(Link::Node(n)) => todo!(),
                Some(Link::Component(c, i)) => {
                    let k = weak_key(c);
                    let h = *self.node_height_table.get(&k).unwrap();
                    let c_position =
                        ComponentPosition::new(x - 1, y - (h as i32) + (*i as i32) + 2);
                    self.node_position_map.insert(k, c_position);
                }
                _ => {}
            }*/
        }
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
