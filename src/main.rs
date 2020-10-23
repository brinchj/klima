#[macro_use]
extern crate horrorshow;

use crate::table::TimeSeriesGroup;
use chrono::NaiveDate;
use std::collections::BTreeMap;

mod dst;
mod table;
mod web;

struct TableFetcher {
    table: String,
    selector: BTreeMap<String, Vec<String>>,
}

impl TableFetcher {
    pub fn new(table: &str) -> TableFetcher {
        TableFetcher {
            table: table.into(),
            selector: Default::default()
        }
    }

    pub fn select(self, key: &str, values: &[&str]) -> Self {
        let mut s = self;
        s.selector.entry(key.into()).or_default().extend(values.iter().map(|s| s.to_string()));
        s
    }

    pub fn fetch(self) -> TimeSeriesGroup {
        let t = dst::Table::new(&self.table).unwrap();
        t.fetch(self.selector).unwrap()
    }
}

fn main() {
    let month = chrono::Duration::days(31);
    let year = chrono::Duration::days(366);

    let cars = TableFetcher::new("BIL51")
        .select("DRIV", &["El"])
        .fetch()
        .accumulative()
        .future_goal(NaiveDate::from_yo(2030, 1), 1_000_000, month);

    let overpost = "Emissioner fra dansk territorium (UNFCCC/UNECE-opgørelsen) (4=(1)÷(2)÷(3))";
    let co2 = "Drivhusgasser i alt, ekskl. CO2 fra afbrænding af biomasse";
    let emissions = TableFetcher::new("MRO2")
        .select("OVERPOST", &[overpost])
        .select("EMTYPE8", &[co2])
        .fetch()
        .future_goal(NaiveDate::from_yo(2030, 1), 21_000, year)
        .future_goal(NaiveDate::from_yo(2050, 1), 0, year);

    println!(
        "{}",
        web::test(web::ChartGraph::bar_plot("CO2e".into(), emissions))
    );
}
