use super::{FieldAnalyzer, ValueType};
use crate::{
    compile_error::{CompileError, CompileErrorType},
    microcontroller::{InputNode, Node, NodeInner, NodeMode, NodePosition, NodeType, OutputNode},
    semantic::evaluate_expr,
    syntax::{MicrocontrollerInterface, MicrocontrollerInterfaceNode, Spanned},
};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

#[derive(Debug)]
pub(super) struct InterfaceAnalyzer<'a> {
    filename: &'a str,
    inputs: bool,
    outputs: bool,
    node_placement: NodePlacement,
}

impl<'a> InterfaceAnalyzer<'a> {
    pub(super) fn new(filename: &'a str, size: Option<(u8, u8)>) -> Self {
        Self {
            filename,
            inputs: false,
            outputs: false,
            node_placement: NodePlacement::new(size),
        }
    }

    pub(super) fn element(
        &mut self,
        element: &Spanned<MicrocontrollerInterface>,
        errors: &mut Vec<CompileError<'a>>,
    ) {
        let (nodes, mode) = match &element.inner {
            MicrocontrollerInterface::Inputs(nodes) => {
                if self.inputs {
                    errors.push(CompileError::new(
                        self.filename,
                        element.span.clone(),
                        CompileErrorType::ElementAlreadyDeclared,
                    ));
                    return;
                } else {
                    self.inputs = true;
                    (nodes, NodeMode::Input)
                }
            }
            MicrocontrollerInterface::Outputs(nodes) => {
                if self.outputs {
                    errors.push(CompileError::new(
                        self.filename,
                        element.span.clone(),
                        CompileErrorType::ElementAlreadyDeclared,
                    ));
                    return;
                } else {
                    self.outputs = true;
                    (nodes, NodeMode::Output)
                }
            }
        };

        for node in nodes {
            match analyze_node(mode, node, self.filename) {
                Ok(n) => self.node_placement.add(n),
                Err(err) => errors.push(err),
            }
        }
    }

    pub(super) fn layout(self) -> Interface {
        let (size, name_nodes) = self.node_placement.layout();

        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        let mut nodes = Vec::with_capacity(name_nodes.len());
        for (name, node) in name_nodes {
            match &node {
                Node::Input(n) => {
                    inputs.insert(name, n.clone());
                }
                Node::Output(n) => {
                    outputs.insert(name, n.clone());
                }
            }
            nodes.push(node);
        }

        Interface {
            size,
            inputs,
            outputs,
            nodes,
        }
    }
}

#[derive(Debug)]
pub(super) struct Interface {
    pub(super) size: (u8, u8),
    pub(super) inputs: HashMap<String, Rc<InputNode>>,
    pub(super) outputs: HashMap<String, Rc<RefCell<OutputNode>>>,
    pub(super) nodes: Vec<Node>,
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

    let mut fields = FieldAnalyzer::new(filename);

    let mut label = None;
    let mut description = None;
    let mut position = None;

    if let Some(f) = &node.fields {
        for assignment in f {
            fields.assignment(assignment, |ident, expr| {
                match ident.as_str() {
                    "name" => label = Some(evaluate_expr(expr, filename)?.try_into()?),
                    "description" => description = Some(evaluate_expr(expr, filename)?.try_into()?),
                    "position" => {
                        position = Some(
                            evaluate_expr(expr, filename)?
                                .tuple_int_ranged(vec![0..=5, 0..=5])?
                                .try_into()?,
                        )
                    }
                    _ => return Ok(false),
                }
                Ok(true)
            })?;
        }
    }

    Ok(FloatingNode {
        name: node.name.clone(),
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
    name: String,
    mode: NodeMode,
    label: String,
    description: String,
    node_type: NodeType,
    position: Option<(u8, u8)>,
}

// ノードの自動配置
#[derive(Debug)]
struct NodePlacement {
    size: Option<(u8, u8)>,
    nodes: VecDeque<FloatingNode>,
    reserved: HashSet<(u8, u8)>,
    count: usize,
}

impl NodePlacement {
    fn new(size: Option<(u8, u8)>) -> Self {
        Self {
            size,
            nodes: VecDeque::new(),
            reserved: HashSet::new(),
            count: 0,
        }
    }

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

    fn layout(mut self) -> ((u8, u8), Vec<(String, Node)>) {
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
                position: NodePosition::new(pos.0, pos.1),
            };
            let n = match node.mode {
                NodeMode::Input => Node::new_input(inner, node.node_type),
                NodeMode::Output => Node::new_output(inner, node.node_type),
            };
            nodes.push((node.name, n));
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
