use crate::web;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::ops::Add;

pub struct TimeSeriesGroup {
    updated: DateTime<Utc>,
    series: Vec<TimeSeries>,
}

impl TimeSeriesGroup {
    pub fn new(updated: DateTime<Utc>, series: Vec<TimeSeries>) -> Self {
        TimeSeriesGroup { updated, series }
    }

    pub fn series(&self) -> &[TimeSeries] {
        &self.series
    }

    pub fn xs(&self) -> im::OrdSet<NaiveDate> {
        self.series
            .iter()
            .flat_map(|f| f.data.keys())
            .cloned()
            .collect()
    }

    pub fn map(self, f: fn(i64) -> i64) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self.series.into_iter().map(|ts| ts.map(f)).collect(),
        }
    }

    pub fn accumulative(self) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.accumulative())
                .collect(),
        }
    }

    pub fn sum(self, title: &str) -> Self {
        self.sum_with_optional_line_color(title, None)
    }

    /// Same as [`sum`](Self::sum), but sets the line/bar color in the chart (otherwise the palette may pick a very dark first color).
    pub fn sum_with_line_color(self, title: &str, rgb: (u8, u8, u8)) -> Self {
        self.sum_with_optional_line_color(title, Some(rgb))
    }

    fn sum_with_optional_line_color(self, title: &str, line_color: Option<(u8, u8, u8)>) -> Self {
        let mut summed = self
            .series
            .into_iter()
            .fold(TimeSeries::default(), std::ops::Add::add)
            .with_tags(im::OrdSet::unit(title.to_string()));
        if let Some(rgb) = line_color {
            summed = summed.with_line_color(rgb);
        }
        TimeSeriesGroup {
            updated: self.updated,
            series: vec![summed],
        }
    }

    pub fn len(&self) -> usize {
        self.series.len()
    }

    pub fn future_goal(
        self,
        title: &str,
        date: NaiveDate,
        goal: i64,
        step: chrono::Duration,
    ) -> Self {
        self.future_goal_with_optional_line_color(title, date, goal, step, None)
    }

    /// Same as [`future_goal`](Self::future_goal), but sets the line colour for this projected series in the chart.
    pub fn future_goal_with_line_color(
        self,
        title: &str,
        date: NaiveDate,
        goal: i64,
        step: chrono::Duration,
        rgb: (u8, u8, u8),
    ) -> Self {
        self.future_goal_with_optional_line_color(title, date, goal, step, Some(rgb))
    }

    fn future_goal_with_optional_line_color(
        self,
        title: &str,
        date: NaiveDate,
        goal: i64,
        step: chrono::Duration,
        line_color: Option<(u8, u8, u8)>,
    ) -> Self {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        let final_date = self.series.iter().map(last_date).max().unwrap();

        let datapoint = |ts: &TimeSeries| *ts.data.get(&final_date).unwrap_or(&0);
        let final_sum: i64 = self.series.iter().map(datapoint).sum();

        let mut running_date = final_date;
        let all_days = (date - running_date).num_days();

        let mut goal_data = im::OrdMap::new();
        while running_date < date {
            running_date = (running_date + step).with_day(1).unwrap();

            let days_spent = (running_date - final_date).num_days();
            let progress = ((goal - final_sum) * days_spent) / all_days;
            goal_data.insert(running_date, final_sum + progress);
        }

        let tags = im::OrdSet::unit(title.to_string());
        let mut goal_series = TimeSeries::new(tags, goal_data);
        if let Some(rgb) = line_color {
            goal_series = goal_series.with_line_color(rgb);
        }
        let mut series = self.series;
        series.push(goal_series);

        TimeSeriesGroup {
            updated: self.updated,
            series,
        }
    }

    pub fn plot(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        let y = format!("{} — {}", y, self.updated.naive_utc());
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y, self)
    }
}

#[derive(Default, Clone)]
pub struct TimeSeries {
    pub tags: im::OrdSet<String>,
    pub data: im::OrdMap<NaiveDate, i64>,
    /// When set, the chart uses this RGB instead of the automatic palette.
    pub line_color_rgb: Option<(u8, u8, u8)>,
}

impl TimeSeries {
    pub fn new(tags: im::OrdSet<String>, data: im::OrdMap<NaiveDate, i64>) -> TimeSeries {
        TimeSeries {
            tags,
            data,
            line_color_rgb: None,
        }
    }

    pub fn unit(tags: im::OrdSet<String>, date: NaiveDate, value: i64) -> TimeSeries {
        TimeSeries {
            tags,
            data: im::OrdMap::unit(date, value),
            line_color_rgb: None,
        }
    }

    pub fn with_tags(self, tags: im::OrdSet<String>) -> Self {
        TimeSeries {
            tags,
            data: self.data,
            line_color_rgb: self.line_color_rgb,
        }
    }

    pub fn with_line_color(self, rgb: (u8, u8, u8)) -> Self {
        TimeSeries {
            line_color_rgb: Some(rgb),
            ..self
        }
    }

    pub fn accumulative(self) -> Self {
        let init = (0i64, im::OrdMap::new());
        let (_total, data) = self
            .data
            .into_iter()
            .fold(init, |(running_total, out), (t, y)| {
                ((y + running_total), out.update(t, y + running_total))
            });
        TimeSeries {
            tags: self.tags.clone(),
            data,
            line_color_rgb: self.line_color_rgb,
        }
    }

    pub fn map(self, f: fn(i64) -> i64) -> Self {
        TimeSeries {
            tags: self.tags,
            data: self.data.into_iter().map(|(k, v)| (k, f(v))).collect(),
            line_color_rgb: self.line_color_rgb,
        }
    }
}

impl Add for TimeSeries {
    type Output = TimeSeries;

    fn add(self, rhs: Self) -> Self::Output {
        TimeSeries {
            tags: self.tags.union(rhs.tags),
            data: self.data.union_with(rhs.data, std::ops::Add::add),
            line_color_rgb: self.line_color_rgb.or(rhs.line_color_rgb),
        }
    }
}
