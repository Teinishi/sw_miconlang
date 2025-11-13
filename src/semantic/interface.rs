use super::{MutField, ValueType, analyze_field};
use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{
        Node, NodeInner, NodeMode, NodePosition, NodeType, UnpositionedMicrocontroller,
    },
    syntax::{MicrocontrollerInterface, MicrocontrollerInterfaceNode, Spanned},
};

use std::collections::{HashSet, VecDeque};

pub(super) fn analyze_microcontroller_interfaces<'a>(
    items: &[Spanned<MicrocontrollerInterface>],
    mc: &mut UnpositionedMicrocontroller,
    filename: &'a str,
    errors: &mut Vec<CompileError<'a>>,
) {
    let mut node_placement = NodePlacement::default();

    for item in items {
        let (nodes, mode) = match &item.inner {
            MicrocontrollerInterface::Inputs(nodes) => (nodes, NodeMode::Input),
            MicrocontrollerInterface::Outputs(nodes) => (nodes, NodeMode::Output),
        };
        for node in nodes {
            match analyze_node(mode, node, filename) {
                Ok(n) => node_placement.add(n),
                Err(err) => errors.push(err),
            }
        }
    }

    let (size, nodes) = node_placement.layout();
    mc.width = size.0;
    mc.length = size.1;
    mc.nodes = nodes;
}

fn analyze_node<'a>(
    mode: NodeMode,
    node: &Spanned<MicrocontrollerInterfaceNode>,
    filename: &'a str,
) -> Result<FloatingNode, CompileError<'a>> {
    let node_type = match ValueType::from_str(&node.type_name) {
        Ok(ValueType::Bool) => NodeType::Bool,
        Ok(ValueType::Float) => NodeType::Number,
        Ok(found_type) => {
            return Err(CompileError::new(
                filename,
                node.span.clone(),
                CompileErrorType::IncompatibleType {
                    expected_types: vec![ValueType::Bool, ValueType::Float],
                    found_type,
                },
            ));
        }
        Err(err) => {
            return Err(CompileError::new(
                filename,
                node.span.clone(),
                CompileErrorType::UnknownType {
                    type_name: err.to_owned(),
                },
            ));
        }
    };

    let mut label = None;
    let mut description = None;
    let mut position = None;

    if let Some(fields) = &node.fields {
        for assignment in fields {
            analyze_field(assignment, filename, |ident| match ident.as_str() {
                "name" => Some(MutField::String(&mut label)),
                "description" => Some(MutField::String(&mut description)),
                "position" => Some(MutField::TupleTwoRangedU8(&mut position, (0..=5, 0..=5))),
                _ => None,
            })?;
        }
    }

    Ok(FloatingNode {
        mode,
        label: label.unwrap_or_else(|| node.name.clone()),
        description: description.unwrap_or_default(),
        node_type,
        position,
    })
}

// 位置を決める前のノード情報
#[derive(Debug)]
struct FloatingNode {
    mode: NodeMode,
    label: String,
    description: String,
    node_type: NodeType,
    position: Option<(u8, u8)>,
}

// ノードの自動配置
#[derive(Default, Debug)]
struct NodePlacement {
    size: Option<(u8, u8)>,
    nodes: VecDeque<FloatingNode>,
    reserved: HashSet<(u8, u8)>,
    count: usize,
}

impl NodePlacement {
    fn add(&mut self, node: FloatingNode) {
        if let Some(pos) = &node.position {
            if self.reserved.insert(*pos) {
                self.count += 1;
            }
        } else {
            self.count += 1;
        }
        self.nodes.push_back(node);
    }

    fn layout(mut self) -> ((u8, u8), Vec<Node>) {
        let n = self.nodes.len();
        let size = self
            .size
            .unwrap_or_else(|| auto_microcontroller_size(self.count));

        let mut nodes = Vec::with_capacity(n);

        let mut i = 0;
        for node in self.nodes {
            let mut pos = node.position;
            loop {
                if pos.is_some() {
                    break;
                }
                let p = (i % size.0, i / size.0);
                if i >= 36 || !self.reserved.contains(&p) {
                    pos = Some(p);
                    break;
                }
                i += 1;
            }
            let pos = pos.unwrap();
            if !self.reserved.insert(pos) {
                // todo: ノード重複の警告メッセージ
            }

            let inner = NodeInner {
                label: node.label,
                description: node.description,
                node_type: node.node_type,
                position: NodePosition::new(pos.0, pos.1),
            };
            let n = match node.mode {
                NodeMode::Input => Node::new_input(inner),
                NodeMode::Output => Node::new_output(inner),
            };
            nodes.push(n);
        }

        (size, nodes)
    }
}

fn auto_microcontroller_size(n: usize) -> (u8, u8) {
    let n = n.clamp(1, 36) as u8;
    let sizes = (1..=6)
        .filter_map(|w| {
            let h = n.div_ceil(w);
            if (w..=6).contains(&h) {
                Some(((w, h), w * h - n))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let min_penalty = sizes.iter().map(|(_, p)| *p).min().unwrap();
    //sizes.reverse(); // 正方形に寄せる場合
    sizes
        .iter()
        .find_map(|(pos, penalty)| (penalty == &min_penalty).then_some(*pos))
        .unwrap()
}
