use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Category {
    index: BTreeMap<String, u64>,
    label: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Unit {
    base: String,
    decimals: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryWithUnit {
    index: BTreeMap<String, u64>,
    label: BTreeMap<String, String>,
    unit: BTreeMap<String, Unit>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dimension {
    label: String,
    category: Category,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentsCode {
    label: String,
    category: CategoryWithUnit,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Role {
    metric: Vec<String>,
    time: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Dimensions {
    dimension: BTreeMap<String, Dimension>,
    contents_code: ContentsCode,
    id: Vec<String>,
    size: Vec<u64>,
    role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionsStaticKeys {
    id: Vec<String>,
    size: Vec<u64>,
    role: Role,
}

impl<'de> Deserialize<'de> for Dimensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s: BTreeMap<String, serde_json::Value> = BTreeMap::deserialize(deserializer)?;

        let id = Vec::deserialize(s.remove("id").unwrap()).unwrap();
        let size = Vec::deserialize(s.remove("size").unwrap()).unwrap();
        let role = Role::deserialize(s.remove("role").unwrap()).unwrap();
        let contents_code = ContentsCode::deserialize(s.remove("ContentsCode").unwrap()).unwrap();

        let dimension = s
            .into_iter()
            .map(|(k, v)| (k, Dimension::deserialize(v).unwrap()))
            .collect();

        Ok(Dimensions {
            dimension,
            contents_code,
            id,
            size,
            role,
        })
    }
}

fn value<T: Serialize>(value: &T) -> serde_json::Value {
    let json = serde_json::to_string(value).unwrap();
    serde_json::from_str(&json).unwrap()
}

impl Serialize for Dimensions {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut s: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        s.insert("id".to_string(), self.id.clone().into());
        s.insert("size".to_string(), self.size.clone().into());
        s.insert("role".to_string(), value(&self.role));
        s.insert("ContentsCode".to_string(), value(&self.contents_code));

        let mut m = self
            .dimension
            .iter()
            .map(|(k, v)| (k.clone(), value(v)))
            .collect();
        s.append(&mut m);

        s.serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dataset {
    dimension: Dimensions,
    label: String,
    source: String,
    updated: String,
    value: Vec<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatasetContainer {
    dataset: Dataset,
}

#[cfg(test)]
mod tests {
    use super::DatasetContainer;
    use serde::Serialize;

    #[test]
    fn parse() {
        let original_json = include_str!("../../../test/data/dst.data.response.bil51.json");
        let d: DatasetContainer = serde_json::from_str(original_json).unwrap();
        let reproduced_json = serde_json::to_string_pretty(&d).unwrap();
        assert_eq!(reproduced_json.trim(), original_json.trim());
    }
}
