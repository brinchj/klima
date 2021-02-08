#[macro_use]
extern crate horrorshow;

use horrorshow::helper::doctype;
use horrorshow::Template;

use crate::table::TimeSeriesGroup;
use chrono::NaiveDate;
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
    let month = chrono::Duration::days(31);
    let year = chrono::Duration::days(366);

    let electric_cars = TableFetcher::new("BIL51")
        .select("DRIV", &["El"])
        .fetch()
        .accumulative()
        .sum("Ny-registrerede elbiler i alt")
        .future_goal(
            "Vej til Klimarådets 2030 mål på 1+ million elbiler",
            NaiveDate::from_yo(2030, 1),
            1_000_000,
            month,
        )
        .plot(
            "electric_cars",
            "Alle nye elbiler siden 2011",
            "måned",
            "samlet antal indregistrerede elbiler",
        );

    let oil_cars = TableFetcher::new("BIL51")
        .select("DRIV", &["Benzin", "Diesel"])
        .fetch()
        .sum("Ny-registrerede benzin og diesel biler per måned")
        .future_goal(
            "Vej til 2030 stop for benzin og diesel",
            NaiveDate::from_yo(2030, 1),
            0,
            month,
        )
        .plot(
            "oil_cars",
            "Nye Benzin og Diesel biler per måned",
            "måned",
            "nye biler per måned",
        );

    let co2 = "Drivhusgasser i alt, ekskl. CO2 fra afbrænding af biomasse";
    let overpost = "Emissioner fra dansk territorium (UNFCCC/UNECE-opgørelsen) (4=(1)÷(2)÷(3))";

    let emissions = TableFetcher::new("MRO2")
        .select("OVERPOST", &[overpost])
        .select("EMTYPE8", &[co2])
        .fetch()
        .sum("Udledninger fra dansk territorium (UNFCCC/UNECE), i alt, ekskl. CO2 fra afbrænding af biomasse")
        .map(|v| v * 1_000)
        .future_goal("Vej til 2030 mål", NaiveDate::from_yo(2030, 1), 21_000_000, year)
        .future_goal("Vej til 2050 mål", NaiveDate::from_yo(2050, 1), 0, year)
        .plot(
            "emissions",
            "Drivhusgasudledninger fra dansk territorium",
            "år",
            "COe ton",
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
                div(class="container") {
                  div(class="row") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        p(class="mb-0") {
                          : "Kampen om at få elbiler på de danske veje handler først og sidst om Danmarks klimamål. At nå klimalovens 70-procentsmål i 2030 kræver en omstilling af vores transportsektor. Jo færre kilometer der køres med benzin- og dieselbiler, jo bedre er chancen for, at vi når vores klimamål."
                        }
                        footer(class="blockquote-footer text-right") {
                          a(href="https://klimaraadet.dk/da/nyheder/uden-elbiler-naar-vi-ikke-klimamaalet", target="_blank") {
                            : "Klimarådet, oktober 2020"
                          }
                        }
                      }
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : electric_cars
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : oil_cars
                    }
                  }
                  hr {}
                  div(class="row") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        p(class="mb-0") {
                          : "70-procentsmålet skal sikre, at Danmark bliver et foregangsland på klimaområdet. Men selvom målet er krævende, peger tidligere beregninger fra Klimarådet på, at 70 pct. i 2030 og klimaneutralitet senest i 2050 ikke er mere ambitiøst end nødvendigt. Målet svarer nemlig nogenlunde til, hvad der skal til, hvis Danmark skal kunne siges at levere sit bidrag til at begrænse den globale temperaturstigning til 1,5 grader."
                        }
                        footer(class="blockquote-footer text-right") {
                          a(href="https://klimaraadet.dk/da/rapporter/kendte-veje-og-nye-spor-til-70-procents-reduktion", target="_blank") {
                            : "Klimarådet, marts 2020"
                          }
                        }
                      }
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : emissions
                    }
                  }
                }
             }
            }
    };

    println!("{}", html.into_string().unwrap());
}
