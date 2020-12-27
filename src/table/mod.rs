use crate::web;
use chrono;
use chrono::{Datelike, NaiveDate};
use im;
use std::ops::Add;

pub struct TimeSeriesGroup {
    series: Vec<TimeSeries>,
}

impl TimeSeriesGroup {
    pub fn new(series: Vec<TimeSeries>) -> Self {
        TimeSeriesGroup { series }
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

    pub fn accumulative(self) -> Self {
        TimeSeriesGroup {
            series: self
                .series
                .into_iter()
                .map(|ts| ts.accumulative())
                .collect(),
        }
    }

    pub fn sum(self, title: &str) -> Self {
        TimeSeriesGroup {
            series: vec![self
                .series
                .into_iter()
                .fold(TimeSeries::default(), std::ops::Add::add)
                .with_tags(im::OrdSet::unit(title.to_string()))],
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
        let mut series = self.series;
        series.push(TimeSeries::new(tags, goal_data));

        TimeSeriesGroup { series }
    }

    pub fn plot(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y.into(), self)
    }
}

#[derive(Default, Clone)]
pub struct TimeSeries {
    pub tags: im::OrdSet<String>,
    pub data: im::OrdMap<NaiveDate, i64>,
}

impl TimeSeries {
    pub fn new(tags: im::OrdSet<String>, data: im::OrdMap<NaiveDate, i64>) -> TimeSeries {
        TimeSeries { tags, data }
    }

    pub fn unit(tags: im::OrdSet<String>, date: NaiveDate, value: i64) -> TimeSeries {
        TimeSeries {
            tags,
            data: im::OrdMap::unit(date, value),
        }
    }

    pub fn with_tags(self, tags: im::OrdSet<String>) -> Self {
        TimeSeries {
            tags,
            data: self.data,
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
        }
    }
}

impl Add for TimeSeries {
    type Output = TimeSeries;

    fn add(self, rhs: Self) -> Self::Output {
        TimeSeries {
            tags: self.tags.union(rhs.tags),
            data: self.data.union_with(rhs.data, std::ops::Add::add),
        }
    }
}
