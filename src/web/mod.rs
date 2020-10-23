use colorous;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;

use crate::dst::DataPoint;
use crate::table::{TimeSeries, TimeSeriesGroup};
use chrono::NaiveDate;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartDataSet {
    label: String,
    background_color: String,
    border_color: String,
    data: Vec<Option<i64>>,
    fill: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChartData {
    labels: Vec<String>,
    datasets: Vec<ChartDataSet>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChartTitle {
    display: bool,
    text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChartToolTips {
    mode: String,
    intersect: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChartHover {
    mode: String,
    intersect: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartScaleLabel {
    display: bool,
    label_string: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartScale {
    stacked: bool,
    display: bool,
    scale_label: ChartScaleLabel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartScales {
    x_axes: Vec<ChartScale>,
    y_axes: Vec<ChartScale>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChartOptions {
    responsive: bool,
    title: ChartTitle,
    tooltips: ChartToolTips,
    hover: ChartHover,
    scales: ChartScales,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartConfig {
    Type: String,
    data: ChartData,
    options: ChartOptions,
}

pub struct ChartGraph {
    name: String,
    config: ChartConfig,
}

impl ChartGraph {
    pub fn bar_plot(name: String, series: TimeSeriesGroup) -> ChartGraph {
        let xs = series.xs();

        let colors = colorous::TURBO;
        let datasets = series
            .series()
            .iter()
            .enumerate()
            .map(|(n, ts)| {
                let color = colors.eval_rational(n, series.len());
                ChartDataSet {
                    label: ts
                        .tags
                        .iter()
                        .map(|d| d.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    background_color: format!("#{:x}", color),
                    border_color: format!("#{:x}", color),
                    data: xs.iter().map(|x| ts.data.get(x).cloned()).collect(),
                    fill: false,
                }
            })
            .collect();

        let options = ChartOptions {
            responsive: true,
            title: ChartTitle {
                display: true,
                text: "title".into(),
            },
            tooltips: ChartToolTips {
                mode: "index".to_string(),
                intersect: false,
            },
            hover: ChartHover {
                mode: "nearest".to_string(),
                intersect: true,
            },
            scales: ChartScales {
                x_axes: vec![ChartScale {
                    stacked: true,
                    display: true,
                    scale_label: ChartScaleLabel {
                        display: true,
                        label_string: "x label".into(),
                    },
                }],
                y_axes: vec![ChartScale {
                    stacked: true,
                    display: true,
                    scale_label: ChartScaleLabel {
                        display: true,
                        label_string: "y label".into(),
                    },
                }],
            },
        };

        let config = ChartConfig {
            Type: "bar".to_string(),
            data: ChartData {
                labels: xs.iter().map(|s| s.format("%Y-%m").to_string()).collect(),
                datasets,
            },
            options,
        };

        ChartGraph { name, config }
    }
}

pub fn test(graph: ChartGraph) -> String {
    let json = serde_json::to_string_pretty(&graph.config).unwrap();

    let js = format!(
        "
var config = {};
window.onload = function () {{
  var ctx = document.getElementById(\"{}\").getContext(\"2d\");
  window.myGraph{} = new Chart(ctx, config);
}};
",
        json, graph.name, graph.name
    );

    format!("{}", html! {
      : doctype::HTML;
      html {
        head {
            script(src = "https://cdnjs.cloudflare.com/ajax/libs/Chart.js/2.9.4/Chart.min.js") {}
            script(src = "https://cdnjs.cloudflare.com/ajax/libs/chartjs-plugin-annotation/0.5.7/chartjs-plugin-annotation.min.js") {}
         }
         body {
            canvas(id=&graph.name) {}
            script {
              : Raw(&js)
            }
         }
      }
    }).to_string()
}
