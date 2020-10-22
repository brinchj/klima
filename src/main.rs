#[macro_use]
extern crate horrorshow;

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;

mod dst;
mod table;
mod web;

fn main() {
    let selector: BTreeMap<String, Vec<String>> = vec![("DRIV".to_string(), vec!["El".to_string()])].into_iter().collect();

    let t = dst::Table::new("BIL51").unwrap();

    let d = t.fetch(selector)
        .unwrap()
        .into_iter()
        .map(|ts| ts.accumulative())
        .collect();

    println!(
        "{}",
        web::test(web::ChartGraph::bar_plot("Elbiler".into(), d))
    );
}
