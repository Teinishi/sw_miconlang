use super::ComponentObject;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct ComponentStates {
    pub c: Vec<(String, ComponentObject)>,
}

impl<'de> Deserialize<'de> for ComponentStates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct ParentVisitor;

        impl<'de> Visitor<'de> for ParentVisitor {
            type Value = ComponentStates;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of XML elements under <ComponentStates>")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut c = Vec::new();

                while let Some((key, value)) = map.next_entry::<String, ComponentObject>()? {
                    c.push((key, value));
                }

                Ok(ComponentStates { c })
            }
        }

        deserializer.deserialize_map(ParentVisitor)
    }
}

impl Serialize for ComponentStates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.c.len()))?;
        for (tag, child) in &self.c {
            map.serialize_entry(tag, child)?;
        }
        map.end()
    }
}
