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

    let mut d: Vec<TimeSeries> = t
        .fetch(selector)
        .unwrap()
        .into_iter()
        .map(|ts| ts.accumulative())
        .collect();

    let last_date = *d
        .iter()
        .map(|ts| ts.data.iter().last().unwrap().0)
        .max()
        .unwrap();

    let last_sum: i64 = d.iter().map(|ts| ts.data.iter().last().unwrap().1).sum();

    let mut date = last_date;
    let goal_date = NaiveDate::from_ymd(2030, 1, 1);
    let all_days = (goal_date - date).num_days();

    let mut goal_data = im::OrdMap::new();
    while date < goal_date {
        date = date
            .with_month(date.month() + 1)
            .or_else(|| date.with_month(1).and_then(|d| d.with_year(d.year() + 1)))
            .unwrap();

        let days_spent = all_days - (goal_date - date).num_days();
        let progress = ((1_000_000 - last_sum) * days_spent) / all_days;
        goal_data.insert(date, last_sum + progress);
    }
    d.push(TimeSeries::new(
        im::OrdSet::unit("Mål,Total".to_string()),
        goal_data,
    ));

    println!(
        "{}",
        web::test(web::ChartGraph::bar_plot("Elbiler".into(), d))
    );
}
