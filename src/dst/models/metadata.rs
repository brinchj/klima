use serde::{Deserialize, Serialize};

mod dst_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S: Serializer>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}", date.format(FORMAT)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        Utc.datetime_from_str(&String::deserialize(deserializer)?, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

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
pub struct Footnote {
    pub text: String,
    pub mandatory: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub active: bool,
    pub contacts: Vec<Contact>,
    pub description: String,
    pub documentation: Documentation,
    pub footnote: Option<Footnote>,
    pub id: String,
    pub suppressed_data_value: String,
    pub text: String,
    pub unit: String,
    #[serde(with = "dst_date_format")]
    pub updated: chrono::DateTime<chrono::Utc>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataRequest<'a> {
    pub table: &'a str,
}

#[cfg(test)]
mod tests {
    use super::Metadata;

    #[test]
    fn parse() {
        let original_json = include_str!("../../../test/data/dst.metadata.response.bil51.json");
        let m: Metadata = serde_json::from_str(original_json).unwrap();
        let reproduced_json = serde_json::to_string_pretty(&m).unwrap();
        assert_eq!(reproduced_json.trim(), original_json.trim());
    }
}
