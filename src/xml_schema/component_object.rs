use super::ComponentPos;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Default, Clone, Debug)]
pub struct ComponentObject {
    pub id: u32,
    pub attrs: Option<HashMap<String, String>>,

    pub pos: Option<ComponentPos>,
    pub inc: Option<ObjectInput>,
    pub items: Option<ObjectItems>,
    pub in_map: BTreeMap<usize, ObjectInput>,
    pub value_list: Vec<(ObjectValueTag, ObjectValue)>,
}

impl<'de> Deserialize<'de> for ComponentObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct ParentVisitor;
        impl<'de> Visitor<'de> for ParentVisitor {
            type Value = ComponentObject;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a ComponentObject element")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut attrs = HashMap::new();

                let mut pos = None;
                let mut inc = None;
                let mut items = None;
                let mut in_map = BTreeMap::new();
                let mut value_list = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    if let Ok(key) = ObjectValueTag::try_from(key.as_str()) {
                        value_list.push((key, map.next_value()?));
                        continue;
                    }

                    if let Some(i) = key.strip_prefix("in").and_then(|i| i.parse::<usize>().ok()) {
                        if in_map.contains_key(&i) {
                            return Err(A::Error::duplicate_field("in_"));
                        }
                        in_map.insert(i, map.next_value()?);
                        continue;
                    }

                    match key.as_str() {
                        "@id" => {
                            id = Some(map.next_value()?);
                        }
                        k if k.starts_with('@') => {
                            attrs.insert(key, map.next_value()?);
                        }
                        "pos" => {
                            pos = Some(map.next_value()?);
                        }
                        "inc" => {
                            inc = Some(map.next_value()?);
                        }
                        "items" => {
                            items = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(A::Error::unknown_field(
                                &key,
                                &[
                                    "@id",
                                    "pos",
                                    "inc",
                                    "items",
                                    "v",
                                    "n",
                                    "r",
                                    "i",
                                    "min",
                                    "max",
                                    "int",
                                    "any attriute",
                                    "any child starts with \"in\" followed by a number",
                                ],
                            ));
                        }
                    }
                }

                Ok(ComponentObject {
                    id: id.ok_or_else(|| A::Error::missing_field("version"))?,
                    attrs: if attrs.is_empty() { None } else { Some(attrs) },
                    pos,
                    inc,
                    items,
                    in_map,
                    value_list,
                })
            }
        }

        deserializer.deserialize_map(ParentVisitor)
    }
}

impl Serialize for ComponentObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("@id", &self.id)?;
        if let Some(attrs) = &self.attrs {
            for (k, v) in attrs {
                map.serialize_entry(k, v)?;
            }
        }

        if let Some(c) = &self.pos {
            map.serialize_entry("pos", c)?;
        }
        if let Some(c) = &self.inc {
            map.serialize_entry("inc", c)?;
        }
        if let Some(c) = &self.items {
            map.serialize_entry("items", c)?;
        }

        for (i, c) in &self.in_map {
            map.serialize_entry(&format!("in{}", i), c)?;
        }
        for (tag, c) in &self.value_list {
            map.serialize_entry(tag.as_str(), c)?;
        }

        map.end()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectInput {
    #[serde(rename = "@component_id", skip_serializing_if = "Option::is_none")]
    pub component_id: Option<u32>,
    #[serde(rename = "@node_index", skip_serializing_if = "Option::is_none")]
    pub node_index: Option<usize>,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum ObjectValueTag {
    V,
    N,
    R,
    I,
    Min,
    Max,
    Int,
}

impl TryFrom<&str> for ObjectValueTag {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "v" => Ok(Self::V),
            "n" => Ok(Self::N),
            "r" => Ok(Self::R),
            "i" => Ok(Self::I),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "int" => Ok(Self::Int),
            _ => Err("unknown object value tag"),
        }
    }
}

impl ObjectValueTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V => "v",
            Self::N => "n",
            Self::R => "r",
            Self::I => "i",
            Self::Min => "min",
            Self::Max => "max",
            Self::Int => "int",
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectValue {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "@value", skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectItems {
    i: Vec<ObjectItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectItem {
    #[serde(rename = "@l")]
    l: String,

    v: ObjectValue,
}
