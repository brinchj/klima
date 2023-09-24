mod models;

use chrono::NaiveDate;

use crate::table::{TimeSeries, TimeSeriesGroup};
use models::data::{DataRequest, DatasetContainer, Dimension, Dimensions, VariableRequest};
use models::metadata::{Metadata, MetadataRequest};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataPoint {
    pub value: i64,
    pub tags: im::OrdMap<String, String>,
}

impl DataPoint {
    fn join_dimensions_and_data(
        dimensions: &[(&str, Vec<&str>)],
        tags: im::OrdMap<String, String>,
        data: &[i64],
    ) -> im::Vector<DataPoint> {
        match dimensions.split_first() {
            None => im::Vector::unit(DataPoint {
                tags,
                value: data[0],
            }),
            Some(((name, variants), dimensions)) => variants
                .iter()
                .map(|variant| tags.update(name.to_string(), variant.to_string()))
                .fold(im::Vector::new(), |res, tags| {
                    &res + &DataPoint::join_dimensions_and_data(
                        dimensions,
                        tags,
                        &data[res.len()..],
                    )
                }),
        }
    }

    fn labels_sorted_by_index(dimension: &Dimension) -> Vec<&str> {
        let swap = |(a, b)| (b, a);
        let label = |id: &&String| dimension.category.label[id.as_str()].as_str();
        dimension
            .category
            .index
            .iter()
            .map(swap)
            .collect::<im::OrdMap<&u64, &String>>()
            .values()
            .map(label)
            .collect()
    }

    fn from_dimensions_and_data(dimensions: &Dimensions, data: &[i64]) -> Vec<DataPoint> {
        let d: Vec<(&str, Vec<&str>)> = dimensions
            .id
            .iter()
            .filter(|id| !dimensions.role.metric.contains(id))
            .map(|id| {
                (
                    id.as_str(),
                    Self::labels_sorted_by_index(&dimensions.dimension[id]),
                )
            })
            .collect();

        DataPoint::join_dimensions_and_data(&d, im::OrdMap::new(), data)
            .into_iter()
            .collect()
    }

    fn parse_time(s: &str) -> NaiveDate {
        if s.len() == 7 && s.contains('M') {
            NaiveDate::parse_from_str(&format!("{}D01", s), "%YM%mD%d")
        } else {
            NaiveDate::parse_from_str(&format!("{}M01D01", s), "%YM%mD%d")
        }
        .expect("failed to understand date format")
    }

    fn to_timeseries(time_id: &str, data: Vec<DataPoint>) -> Vec<TimeSeries> {
        let tmp: im::OrdMap<im::OrdSet<String>, TimeSeries> =
            data.into_iter().fold(im::OrdMap::new(), |m, p| {
                let time = Self::parse_time(&p.tags[time_id]);
                let tags: im::OrdSet<String> = p.tags.without(time_id).values().collect();
                let new = TimeSeries::unit(tags.clone(), time, p.value);
                m.update_with(tags, new, std::ops::Add::add)
            });
        tmp.into_iter().map(|(_, ts)| ts).collect()
    }
}

pub struct Table {
    client: reqwest::blocking::Client,
    table: String,
    metadata: Metadata,
}

impl Table {
    pub fn new(table: &str) -> Result<Table, failure::Error> {
        let client = reqwest::blocking::Client::new();
        let metadata = client
            .post("https://api.statbank.dk/v1/tableinfo")
            .json(&MetadataRequest { table })
            .send()?
            .json()?;

        Ok(Table {
            client,
            metadata,
            table: table.into(),
        })
    }

    pub fn fetch(
        &self,
        field_selector: BTreeMap<String, Vec<String>>,
    ) -> Result<TimeSeriesGroup, failure::Error> {
        let id_selector: BTreeMap<String, Vec<&str>> = field_selector
            .into_iter()
            .map(|(k, v)| {
                let metadata = self.metadata.variables.iter().find(|v| v.id == k).unwrap();
                let ids: Vec<&str> = v
                    .into_iter()
                    .map(|text| {
                        metadata
                            .values
                            .iter()
                            .find(|v| v.text == text)
                            .unwrap_or_else(|| panic!("no such variable value: {}", text))
                            .id
                            .as_str()
                    })
                    .collect();
                (k, ids)
            })
            .collect();

        let request = DataRequest {
            table: self.table.as_str(),
            format: "JSONSTAT".to_string(),
            variables: self
                .metadata
                .variables
                .iter()
                .map(|v| VariableRequest {
                    code: v.id.as_str(),
                    values: id_selector
                        .get(&v.id)
                        .cloned()
                        .unwrap_or(vec!["*"])
                        .to_owned(),
                })
                .collect(),
        };

        let response: DatasetContainer = self
            .client
            .post("https://api.statbank.dk/v1/data")
            .json(&request)
            .send()?
            .json()?;

        let values: Vec<_> = response
            .dataset
            .value
            .iter()
            .map(|v| v.unwrap_or(0))
            .collect();

        let time_id = self.metadata.variables.iter().find(|v| v.time).unwrap();

        Ok(TimeSeriesGroup::new(
            self.metadata.updated,
            DataPoint::to_timeseries(
                &time_id.id,
                DataPoint::from_dimensions_and_data(&response.dataset.dimension, &values),
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::DataPoint;
    use super::DatasetContainer;
    use super::Metadata;

    #[test]
    fn test_from_dimensions_and_data() {
        let dimensions = vec![
            ("DRIV", vec!["Benzin", "Diesel", "El"]),
            ("EJER", vec!["Privat", "Erhverv"]),
            ("Data", vec!["Data"]),
            ("Tid", vec!["2018", "2019", "2020"]),
        ];

        let mut expect: Vec<DataPoint> = vec![];

        let mut values = vec![];
        for driv in &dimensions[0].1 {
            for ejer in &dimensions[1].1 {
                for data in &dimensions[2].1 {
                    for tid in &dimensions[3].1 {
                        let n = values.len() as i64;
                        values.push(n);
                        expect.push(DataPoint {
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

        let got = DataPoint::join_dimensions_and_data(&dimensions, im::OrdMap::new(), &values);
        assert_eq!(got, expect.into());
    }

    #[test]
    fn test() {
        assert!(serde_json::from_str::<Metadata>(include_str!(
            "../../test/data/dst.metadata.response.bil51.json"
        ))
        .is_ok());

        assert!(serde_json::from_str::<DatasetContainer>(include_str!(
            "../../test/data/dst.data.response.bil51.large.json"
        ))
        .is_ok());
    }
}
