mod models;

use im;

use models::data::DatasetContainer;
use models::metadata::{Metadata, Value, Variable};
use std::collections::BTreeMap;

pub struct VarKey(String);
pub struct ValKey(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataPoint {
    value: i64,
    tags: im::OrdMap<String, String>
}

impl DataPoint {
    fn join_dimensions_and_data(
        dimensions: &[(&str, Vec<&str>)],
        tags: im::OrdMap<String, String>,
        data: &[i64],
    ) -> im::Vector<DataPoint> {
        match dimensions.split_first() {
            None => im::Vector::unit(DataPoint{tags, value: data[0]}),
            Some(((name, variants), dimensions)) => {
                variants.iter().map(|variant|
                    tags.update(name.to_string(), variant.to_string())
                ).fold(im::Vector::new(), |res, tags| {
                    &res + &DataPoint::join_dimensions_and_data(dimensions, tags, &data[res.len()..])
                })
            }
        }
    }

    fn from_dimensions_and_data(dimensions: &[(&str, Vec<&str>)], data: &[i64]) -> Vec<DataPoint> {
        return DataPoint::join_dimensions_and_data(dimensions, im::OrdMap::new(), data)
            .into_iter()
            .collect()
    }
}

fn variable<'a>(metadata: &'a Metadata, var: &VarKey) -> Option<&'a Variable> {
    metadata.variables.iter().find(|v| v.id == var.0)
}

fn variable_code<'a>(metadata: &'a Metadata, var: &VarKey, val: &ValKey) -> Option<&'a Value> {
    variable(metadata, var).and_then(|v| v.values.iter().find(|v| v.text == val.0))
}





#[cfg(test)]
mod tests {
    use super::DataPoint;
    use super::DatasetContainer;
    use super::Metadata;
    use std::collections::BTreeMap;

    #[test]
    fn test_from_dimensions_and_data() {
        let dimensions = vec![
            ("DRIV", vec!["Benzin", "Diesel", "El"]),
            ("EJER", vec!["Privat", "Erhverv"]),
            ("Data", vec!["Data"]),
            ("Tid", vec!["2018", "2019", "2020"]),
        ];

        let mut m: Vec<DataPoint> = vec![];

        let mut values = vec![];
        for driv in &dimensions[0].1 {
            for ejer in &dimensions[1].1 {
                for data in &dimensions[2].1 {
                    for tid in &dimensions[3].1 {
                        let n = values.len() as i64;
                        values.push(n);
                        m.push(DataPoint {
                            tags: vec![
                                ("DRIV".to_string(), driv.to_string()),
                                ("EJER".to_string(), ejer.to_string()),
                                ("Data".to_string(), data.to_string()),
                                ("Tid".to_string(), tid.to_string()),
                            ]
                                .into_iter()
                                .collect(),
                            value: n,
                        });
                    }
                }
            }
        }

        assert_eq!(
            DataPoint::from_dimensions_and_data(&dimensions, &values), m
        );
    }

    #[test]
    fn test() {
        let metadata: Metadata = serde_json::from_str(include_str!(
            "../../test/data/dst.metadata.response.bil51.json"
        ))
        .unwrap();
        let data: DatasetContainer = serde_json::from_str(include_str!(
            "../../test/data/dst.data.response.bil51.large.json"
        ))
        .unwrap();
    }
}
