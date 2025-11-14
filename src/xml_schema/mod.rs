mod attrs;
mod component_object;
mod component_states;
pub mod conversion;
pub use attrs::Attrs;
pub use component_object::{ComponentObject, ObjectValue, ObjectValueTag};
pub use component_states::ComponentStates;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "microprocessor")]
pub struct Microprocessor {
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "@width")]
    pub width: u8,
    #[serde(rename = "@length")]
    pub length: u8,
    #[serde(rename = "@id_counter")]
    pub id_counter: u32,
    #[serde(rename = "@id_counter_node")]
    pub id_counter_node: u32,
    #[serde(flatten)]
    pub attrs: Attrs,

    pub nodes: Nodes,
    pub group: Group,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Nodes {
    #[serde(default)]
    pub n: Vec<NodeItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NodeItem {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@component_id")]
    pub component_id: u32,

    pub node: Node,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "@mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<u8>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub node_type: Option<u8>,
    #[serde(rename = "@description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<NodePos>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NodePos {
    #[serde(rename = "@x", skip_serializing_if = "Option::is_none")]
    pub x: Option<u8>,
    #[serde(rename = "@z", skip_serializing_if = "Option::is_none")]
    pub z: Option<u8>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub data: GroupData,
    pub components: Components,
    pub components_bridge: Components,
    pub groups: Groups,
    pub component_states: ComponentStates,
    pub component_bridge_states: ComponentStates,
    pub group_states: GroupStates,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupData {
    pub inputs: GroupDataInputs,
    pub outputs: GroupDataOutputs,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupDataInputs;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupDataOutputs;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Groups;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupStates;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Components {
    #[serde(default)]
    pub c: Vec<ComponentItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComponentItem {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub component_type: Option<u8>,

    pub object: ComponentObject,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComponentPos {
    #[serde(rename = "@x", skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(rename = "@y", skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
}
