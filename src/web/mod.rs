use colorous;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::table::TimeSeriesGroup;
use horrorshow::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartDataSet {
    label: String,
    background_color: String,
    border_color: String,
    data: Vec<Option<i64>>,
    fill: String,
    border_width: u64,
    point_radius: u64,
    point_hover_radius: u64,
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
    type_: String,
    data: ChartData,
    options: ChartOptions,
}

pub struct ChartGraph {
    name: String,
    config: ChartConfig,
}

impl ChartGraph {
    pub fn bar_plot(
        id: String,
        title: String,
        x: String,
        y: String,
        series: TimeSeriesGroup,
    ) -> ChartGraph {
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
                    fill: "start".to_string(),
                    border_width: 1,
                    point_radius: 0,
                    point_hover_radius: 1,
                }
            })
            .collect();

        let options = ChartOptions {
            responsive: true,
            title: ChartTitle {
                display: false,
                text: title,
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
                        display: false,
                        label_string: x,
                    },
                }],
                y_axes: vec![ChartScale {
                    stacked: true,
                    display: true,
                    scale_label: ChartScaleLabel {
                        display: true,
                        label_string: y,
                    },
                }],
            },
        };

        let config = ChartConfig {
            type_: "line".to_string(),
            data: ChartData {
                labels: xs.iter().map(|s| s.format("%Y-%m").to_string()).collect(),
                datasets,
            },
            options,
        };

        ChartGraph { name: id, config }
    }

    pub fn bar_plot_html(
        id: String,
        title: String,
        x: String,
        y: String,
        series: TimeSeriesGroup,
    ) -> impl horrorshow::RenderOnce {
        let graph = Self::bar_plot(id.clone(), title, x, y, series);
        let json = serde_json::to_string_pretty(&graph.config).unwrap();

        let js = format!(
            "
window.addEventListener(\"load\", function () {{
  var config = {};
  var ctx = document.getElementById(\"{}\").getContext(\"2d\");
  window.myGraph{} = new Chart(ctx, config);
}});",
            json, graph.name, graph.name
        );

        html! {
            canvas(id=id) {}
            script {
              : Raw(js)
            }
        }
    }
}
