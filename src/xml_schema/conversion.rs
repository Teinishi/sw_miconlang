use super::{
    Attrs, ComponentStates, Components, Group, Microprocessor, Node, NodeItem, NodePos, Nodes,
};
use crate::{
    microcontroller::{self, Link, PositionedMicrocontroller},
    xml_schema::component_object::ObjectInput,
};

use std::{collections::HashMap, rc::Rc};

// デフォルトなら None 、それ以外なら Some でラップ
fn to_option<T: PartialEq>(value: T, default: T) -> Option<T> {
    if value == default { None } else { Some(value) }
}

fn option_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn option_u8(value: u8) -> Option<u8> {
    to_option(value, 0)
}

fn option_u32(value: u32) -> Option<u32> {
    to_option(value, 0)
}

fn option_usize(value: usize) -> Option<usize> {
    to_option(value, 0)
}

fn option_node_pos(value: microcontroller::NodePosition) -> Option<NodePos> {
    if value.x == 0 && value.z == 0 {
        None
    } else {
        Some(NodePos {
            x: option_u8(value.x),
            z: option_u8(value.z),
        })
    }
}

#[derive(Debug)]
pub enum MicroprocessorConversionError {
    UnknownInputNode,
    UnknownInputComponent,
}

// コンポーネントのIDをカウント、割り当て、リンクからID取得
#[derive(Default, Debug)]
struct ComponentIdManager {
    id_counter: u32,
    input_node_id_map: HashMap<usize, u32>,
    id_map: HashMap<usize, u32>,
}

impl ComponentIdManager {
    fn generate_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }

    fn add_node(&mut self, node: &microcontroller::Node) -> u32 {
        let id = self.generate_id();
        if let microcontroller::Node::Input(n) = node {
            self.input_node_id_map.insert(Rc::as_ptr(n) as usize, id);
        }
        id
    }

    fn add(&mut self, component: &Rc<microcontroller::Component>) -> u32 {
        let id = self.generate_id();
        self.id_map.insert(Rc::as_ptr(component) as usize, id);
        id
    }

    fn get_object_input(&self, link: &Link) -> Result<ObjectInput, MicroprocessorConversionError> {
        match link {
            Link::Node(n) => Ok(ObjectInput {
                component_id: self
                    .input_node_id_map
                    .get(&(n.as_ptr() as usize))
                    .map(|value: &u32| option_u32(*value))
                    .ok_or(MicroprocessorConversionError::UnknownInputNode)?,
                node_index: None,
            }),
            Link::Component(c, i) => Ok(ObjectInput {
                component_id: self
                    .id_map
                    .get(&(c.as_ptr() as usize))
                    .map(|value: &u32| option_u32(*value))
                    .ok_or(MicroprocessorConversionError::UnknownInputComponent)?,
                node_index: option_usize(*i),
            }),
        }
    }
}

impl TryFrom<&PositionedMicrocontroller> for Microprocessor {
    type Error = MicroprocessorConversionError;

    fn try_from(value: &PositionedMicrocontroller) -> Result<Self, Self::Error> {
        let num_nodes = value.nodes.len();
        let num_components = value.components.len();

        let mut node_id_counter = 0;

        // コンポーネントへの Weak 参照からIDを取得
        let mut id_manager = ComponentIdManager {
            id_counter: 0,
            input_node_id_map: HashMap::new(),
            id_map: HashMap::with_capacity(num_components),
        };

        // マイコン入出力ノード
        let mut nodes = Vec::with_capacity(num_nodes);
        let mut node_components = Vec::with_capacity(num_nodes);
        for node in &value.nodes {
            node_id_counter += 1;

            // InputNode へのポインタ -> ID を記録
            let id = id_manager.add_node(&node.inner);

            // <nodes> に追加
            nodes.push(NodeItem {
                id: node_id_counter,
                component_id: id,
                node: Node {
                    label: option_string(node.label_owned()),
                    mode: option_u8(node.mode().into()),
                    node_type: option_u8(node.node_type().into()),
                    description: option_string(node.description_owned()),
                    position: option_node_pos(node.position()),
                },
            });

            // <components_bridge> に追加
            node_components.push(node.as_xml_item(id));
        }

        // コンポーネント
        let mut components = Vec::with_capacity(num_components);
        for component in &value.components {
            // Component へのポインタ -> ID を記録
            let id = id_manager.add(&component.inner);

            // <components> に追加
            components.push(component.as_xml_item(id));
        }

        // コンポーネントの入力接続
        let mut component_states = Vec::with_capacity(num_components);
        for (i, (component, item)) in value
            .components
            .iter()
            .zip(components.iter_mut())
            .enumerate()
        {
            for (j, link) in component.input_links().iter().enumerate() {
                if let Some(link) = link {
                    item.object
                        .in_map
                        .insert(j + 1, id_manager.get_object_input(link)?);
                }
            }

            // <component_states> に追加
            component_states.push((format!("c{}", i), item.object.clone()));
        }

        // 出力ノードへの入力接続
        let mut component_bridge_states = Vec::with_capacity(num_nodes);
        for (i, (node, item)) in value
            .nodes
            .iter()
            .zip(node_components.iter_mut())
            .enumerate()
        {
            if let microcontroller::Node::Output(n) = &node.inner
                && let Some(link) = &n.borrow().input
            {
                item.object
                    .in_map
                    .insert(1, id_manager.get_object_input(link)?);
            }

            // <component_bridge_states> に追加
            component_bridge_states.push((format!("c{}", i), item.object.clone()));
        }

        Ok(Self {
            name: option_string(value.name.clone()),
            description: option_string(value.description.clone()),
            width: value.width,
            length: value.length,
            id_counter: id_manager.id_counter,
            id_counter_node: node_id_counter,
            attrs: Attrs::default(),
            nodes: Nodes { n: nodes },
            group: Group {
                components: Components { c: components },
                components_bridge: Components { c: node_components },
                component_states: ComponentStates {
                    c: component_states,
                },
                component_bridge_states: ComponentStates {
                    c: component_bridge_states,
                },
                ..Default::default()
            },
        })
    }
}
