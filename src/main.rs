#[macro_use]
extern crate horrorshow;

use serde::{Deserialize, Serialize};
use serde_json;

mod dst;
mod web;

fn main() {
    let t = dst::Table::new("BIL51").unwrap();
    let d = t.fetch_all().unwrap();
    println!("{}", web::test(web::ChartGraph::new("Elbiler".into(), "Tid".into(), d)));
}
