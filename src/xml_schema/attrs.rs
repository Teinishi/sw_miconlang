use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Attrs {
    inner: Option<HashMap<String, String>>,
}

impl<'de> Deserialize<'de> for Attrs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        if raw.is_empty() {
            return Ok(Self { inner: None });
        }

        let attrs = raw
            .into_iter()
            .filter(|(k, _)| k.starts_with('@'))
            .map(|(k, v)| (k.trim_start_matches('@').to_string(), v))
            .collect();
        Ok(Self {
            inner: Some(attrs),
        })
    }
}

impl Serialize for Attrs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let map = if let Some(other_attrs) = &self.inner {
            // キーでソートしてからシリアライズ
            let mut entries = other_attrs.iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| natord::compare(a.0, b.0));

            let mut map = serializer.serialize_map(Some(entries.len()))?;
            for (k, v) in entries {
                let key = if k.starts_with('@') {
                    k.clone()
                } else {
                    format!("@{}", k)
                };
                map.serialize_entry(&key, v)?;
            }
            map
        } else {
            serializer.serialize_map(Some(0))?
        };
        map.end()
    }
}
