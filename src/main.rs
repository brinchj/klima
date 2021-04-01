#[macro_use]
extern crate horrorshow;

use horrorshow::helper::doctype;
use horrorshow::Template;

use crate::table::TimeSeriesGroup;
use chrono::{NaiveDate, Datelike, Weekday};
use std::collections::BTreeMap;

mod dst;
mod table;
mod web;

struct TableFetcher {
    table: dst::Table,
    selector: BTreeMap<String, Vec<String>>,
}

impl TableFetcher {
    pub fn new(table: &str) -> TableFetcher {
        TableFetcher {
            table: dst::Table::new(table).unwrap(),
            selector: Default::default(),
        }
    }

    pub fn select(self, key: &str, values: &[&str]) -> Self {
        let mut s = self;
        s.selector
            .entry(key.into())
            .or_default()
            .extend(values.iter().map(|s| s.to_string()));
        s
    }

    pub fn fetch(self) -> TimeSeriesGroup {
        self.table.fetch(self.selector).unwrap()
    }
}

fn main() {
    // let month = chrono::Duration::days(31);
    // let year = chrono::Duration::days(366);

    let alder_90 = TableFetcher::new("DODC1")
        .select("KØN", &["I alt"])
        .select("ALDER", &["90-94 år", "95-99 år", "100 år og derover"])
        .fetch()
        .right(&NaiveDate::from_ymd(2020, 9, 8))
        .sum("90+ år")
        .reduce(|d| d.weekday() == Weekday::Mon, |a, b| a + b)
        .plot(
            "alder_90",
            "Covid-relaterede dødsfald per uge for borgere over 90 år",
            "uge",
            "antal dødsfald",
        );

    let smit_90 = TableFetcher::new("SMIT2")
        .select("AKTP", &["Bekræftede COVID-19 tilfælde pr. 100.000 personer"])
        .select("ALDER1", &["90 år og derover", "80-89 år", "70-79 år", "60-69 år"])
        .fetch()
        .right(&NaiveDate::from_ymd(2020, 9, 8))
        .reduce(|d| d.weekday() == Weekday::Mon, |a, b| b - a)
        .norm("60-69 år")
        .plot(
            "smit_90",
            "Covid smittede per uge for borgere over 90 år",
            "uge",
            "antal registreret smittet",
        );

    let html = html! {
          : doctype::HTML;
          html {
            head {
                link(rel="stylesheet", href="https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/css/bootstrap.min.css") {}
                script(src = "https://code.jquery.com/jquery-3.5.1.slim.min.js", integrity="sha384-DfXdz2htPH0lsSSs5nCTpuj/zy4C+OGpamoFVy38MVBnE+IbbVYUew+OrCXaRkfj", crossorigin="anonymous") {}
                script(src = "https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/js/bootstrap.bundle.min.js", integrity="sha384-ho+j7jyWK8fNQe+A12Hb8AhRq26LrZ/JpcUGGOn+Y7RsweNrtN/tE3MoK7ZeZDyx", crossorigin="anonymous") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/Chart.js/2.9.4/Chart.min.js") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/chartjs-plugin-annotation/0.5.7/chartjs-plugin-annotation.min.js") {}
             }
             body {
                  // div(class="row") {
                  //   div(class="col col-lg-12") {
                  //     : alder_90
                  //   }
                  // }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : smit_90
                    }
                  }
                }
              }
    };

    println!("{}", html.into_string().unwrap());
}
