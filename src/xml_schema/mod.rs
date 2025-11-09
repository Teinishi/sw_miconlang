mod attrs;
mod component_object;
mod component_states;
pub use attrs::Attrs;
pub use component_object::ComponentObject;
pub use component_states::ComponentStates;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "microprocessor")]
pub struct Microprocessor {
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "@description", skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "@width")]
    width: u32,
    #[serde(rename = "@length")]
    length: u32,
    #[serde(rename = "@id_counter")]
    id_counter: u32,
    #[serde(rename = "@id_counter_node")]
    id_ccounter_ndoe: u32,
    #[serde(flatten)]
    attrs: Attrs,

    nodes: Nodes,
    group: Group,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Nodes {
    #[serde(default)]
    n: Vec<NodeItem>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NodeItem {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "@component_id")]
    component_id: u32,

    node: Node,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(rename = "@mode", skip_serializing_if = "Option::is_none")]
    mode: Option<u32>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    node_type: Option<u32>,
    #[serde(rename = "@description", skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<Position>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Position {
    #[serde(rename = "@x", skip_serializing_if = "Option::is_none")]
    x: Option<i32>,
    #[serde(rename = "@y", skip_serializing_if = "Option::is_none")]
    y: Option<i32>,
    #[serde(rename = "@z", skip_serializing_if = "Option::is_none")]
    z: Option<i32>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Group {
    data: GroupData,
    components: Components,
    components_bridge: Components,
    groups: Groups,
    component_states: ComponentStates,
    component_bridge_states: ComponentStates,
    group_states: GroupStates,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupData {
    inputs: GroupDataInputs,
    outputs: GroupDataOutputs,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupDataInputs;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupDataOutputs;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Groups;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupStates;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Components {
    #[serde(default)]
    c: Vec<ComponentItem>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComponentItem {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    component_type: Option<u32>,

    object: ComponentObject,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComponentPos {
    #[serde(rename = "@x", skip_serializing_if = "Option::is_none")]
    x: Option<f32>,
    #[serde(rename = "@y", skip_serializing_if = "Option::is_none")]
    y: Option<f32>,
}
