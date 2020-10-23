#[macro_use]
extern crate horrorshow;

use horrorshow::Template;
use horrorshow::helper::doctype;

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

    let co2 = "Drivhusgasser i alt, ekskl. CO2 fra afbrænding af biomasse";
    let bio = "Kuldioxid (CO2) fra afbrænding af biomasse";

    let overpost = "Emissioner fra dansk territorium (UNFCCC/UNECE-opgørelsen) (4=(1)÷(2)÷(3))";
    let international_transport = "Emissioner i udlandet (international transport) (2)=(2.1)+(2.2)+(2.3)";

    let emissions = TableFetcher::new("MRO2")
        .select("OVERPOST", &[overpost])
        .select("EMTYPE8", &[co2])
        .fetch()
        .future_goal(NaiveDate::from_yo(2030, 1), 21_000, year)
        .future_goal(NaiveDate::from_yo(2050, 1), 0, year);

    let i_transport = TableFetcher::new("MRO2")
        .select("OVERPOST", &[international_transport])
        .select("EMTYPE8", &[co2])
        .fetch()
        .future_goal(NaiveDate::from_yo(2050, 1), 20_000, year);

    let bio = TableFetcher::new("MRO2")
        .select("OVERPOST", &[overpost])
        .select("EMTYPE8", &[bio])
        .fetch();

    let cars_html = web::ChartGraph::bar_plot_html("cars".into(), cars);

    let emissions_html = web::ChartGraph::bar_plot_html("emissions".into(), emissions);

    let transport_html = web::ChartGraph::bar_plot_html("transport".into(), i_transport);

    let bio_html = web::ChartGraph::bar_plot_html("bio".into(), bio);

    let html = html! {
          : doctype::HTML;
          html {
            head {
                link(rel="stylesheet", href="https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/css/bootstrap.min.css") {}
                script(src = "https://code.jquery.com/jquery-3.5.1.slim.min.js", integrity="sha384-DfXdz2htPH0lsSSs5nCTpuj/zy4C+OGpamoFVy38MVBnE+IbbVYUew+OrCXaRkfj", crossorigin="anonymous") {}
                script(src = "https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/js/bootstrap.bundle.min.js", integrity="sha384-ho+j7jyWK8fNQe+A12Hb8AhRq26LrZ/JpcUGGOn+Y7RsweNrtN/tE3MoK7ZeZDyx", crossorigin="anonymous") {}
                script(src = "https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/js/bootstrap.bundle.min.js") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/Chart.js/2.9.4/Chart.min.js") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/chartjs-plugin-annotation/0.5.7/chartjs-plugin-annotation.min.js") {}
             }
             body {
                div(class="container") {
                  div(class="row") {
                    div(class="col col-lg-6") {
                      : cars_html
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-6") {
                      : emissions_html
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-6") {
                      : transport_html
                    }
                  }
                }
             }
            }
    };

    println!("{}", html.into_string().unwrap());
}
