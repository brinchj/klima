#[macro_use]
extern crate horrorshow;

use crate::table::TimeSeries;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;

mod dst;
mod table;
mod web;

fn main() {
    let selector: BTreeMap<String, Vec<String>> =
        vec![("DRIV".to_string(), vec!["El".to_string()])]
            .into_iter()
            .collect();

    let t = dst::Table::new("BIL51").unwrap();

    let mut d = t
        .fetch(selector)
        .unwrap()
        .accumulative()
        .future_goal(NaiveDate::from_yo(2030, 1), 1_000_000);

    println!(
        "{}",
        web::test(web::ChartGraph::bar_plot("Elbiler".into(), d))
    );
}
