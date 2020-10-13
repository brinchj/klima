mod models;

use models::data::DatasetContainer;
use models::metadata::{Metadata, Value, Variable};
use std::collections::BTreeMap;

pub struct VarKey(String);
pub struct ValKey(String);

fn variable<'a>(metadata: &'a Metadata, var: &VarKey) -> Option<&'a Variable> {
    metadata.variables.iter().find(|v| v.id == var.0)
}

fn variable_code<'a>(metadata: &'a Metadata, var: &VarKey, val: &ValKey) -> Option<&'a Value> {
    variable(metadata, var).and_then(|v| v.values.iter().find(|v| v.text == val.0))
}

fn join_dimensions_and_data(
    dimensions: &[(&str, Vec<&str>)],
    context: &mut BTreeMap<String, String>,
    data: &[i64],
) -> Vec<(BTreeMap<String, String>, i64)> {
    return match dimensions.split_first() {
        None => vec![(context.clone(), data[0])],
        Some(((name, variants), tail)) => {
            let mut res = vec![];
            for variant in variants {
                context.insert(name.to_string(), variant.to_string());
                res.append(&mut join_dimensions_and_data(
                    tail,
                    context,
                    &data[res.len()..],
                ))
            }
            res
        }
    };
}

#[cfg(test)]
mod tests {
    use super::join_dimensions_and_data;
    use super::DatasetContainer;
    use super::Metadata;
    use std::collections::BTreeMap;

    #[test]
    fn test_join_dimensions_and_data() {
        let dimensions = vec![
            ("DRIV", vec!["Benzin", "Diesel", "El"]),
            ("EJER", vec!["Privat", "Erhverv"]),
            ("Data", vec!["Data"]),
            ("Tid", vec!["2018", "2019", "2020"]),
        ];

        let mut m: Vec<(BTreeMap<String, String>, i64)> = vec![];

        let mut values = vec![];
        for driv in &dimensions[0].1 {
            for ejer in &dimensions[1].1 {
                for data in &dimensions[2].1 {
                    for tid in &dimensions[3].1 {
                        let n = values.len() as i64;
                        values.push(n);
                        m.push((
                            vec![
                                ("DRIV".to_string(), driv.to_string()),
                                ("EJER".to_string(), ejer.to_string()),
                                ("Data".to_string(), data.to_string()),
                                ("Tid".to_string(), tid.to_string()),
                            ]
                            .into_iter()
                            .collect(),
                            n,
                        ));
                    }
                }
            }
        }

        let mut context = BTreeMap::new();
        assert_eq!(
            join_dimensions_and_data(&dimensions, &mut context, &values),
            m
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
