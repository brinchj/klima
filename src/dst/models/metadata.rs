use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub mail: String,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Documentation {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Variable {
    pub elimination: bool,
    pub id: String,
    pub text: String,
    pub time: bool,
    pub values: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub active: bool,
    pub contacts: Vec<Contact>,
    pub description: String,
    pub documentation: Documentation,
    pub footnote: Option<String>,
    pub id: String,
    pub suppressed_data_value: String,
    pub text: String,
    pub unit: String,
    pub updated: String,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataRequest<'a> {
    pub table: &'a str,
}

#[cfg(test)]
mod tests {
    use super::Metadata;
    use serde::Serialize;

    #[test]
    fn parse() {
        let original_json = include_str!("../../../test/data/dst.metadata.response.bil51.json");
        let m: Metadata = serde_json::from_str(original_json).unwrap();
        let reproduced_json = serde_json::to_string_pretty(&m).unwrap();
        assert_eq!(reproduced_json.trim(), original_json.trim());
    }
}
