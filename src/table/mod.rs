use crate::web;
use chrono;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use im;
use std::ops::{Add, Div};

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
            series: self.series.into_iter().map(|ts| ts.map(f)).collect()
        }
    }

    fn final_date(&self) -> NaiveDate {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        self.series.iter().map(last_date).max().unwrap()
    }

    pub fn accumulative(self) -> Self {
        let final_date = self.final_date();
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.accumulative(final_date))
                .collect(),
        }
    }

    pub fn right(self, from: &NaiveDate) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.right(from))
                .collect(),
        }
    }

    pub fn reduce(self, when: fn(&NaiveDate) -> bool, how: fn(i64, i64) -> i64) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.reduce(when, how))
                .collect(),
        }
    }

    pub fn norm(self, tag: &str) -> Self {
        let index = self.series.iter().find(|ts| ts.tags.contains(tag)).unwrap().clone();
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .filter(|ts| !ts.tags.contains(tag))
                .map(|ts| ts / index.clone())
                .collect(),
        }
    }

    pub fn sum(self, title: &str) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
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
        let final_date = self.final_date();

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

        TimeSeriesGroup {
            updated: self.updated,
            series,
        }
    }

    pub fn plot(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        let y = format!("{} — {}", y, self.updated.date().naive_local().to_string());
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y, self)
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

    pub fn accumulative(self, final_date: NaiveDate) -> Self {
        let init = (0i64, im::OrdMap::new());

        let (total, mut data) = self
            .data
            .into_iter()
            .fold(init, |(running_total, out), (t, y)| {
                ((y + running_total), out.update(t, y + running_total))
            });

        if !data.contains_key(&final_date) {
            data.insert(final_date, total);
        }

        TimeSeries {
            tags: self.tags.clone(),
            data,
        }
    }

    pub fn reduce(self, when: fn(&NaiveDate) -> bool, how: fn(i64, i64) -> i64) -> Self {
        let k = self.data.keys().min().unwrap();
        let first = *self.data.get(k).unwrap();
        let init = (0i64, first, im::OrdMap::new());
        let (_total, _prev, data) = self
            .data
            .into_iter()
            .fold(init, |(running_total, prev, out), (t, y)| {
                let delta = how(prev, y);
                if when(&t) {
                    (delta, y, out.update(t, running_total))
                } else {
                    (running_total + delta, y, out)
                }
            });
        TimeSeries {
            tags: self.tags.clone(),
            data,
        }
    }

    pub fn right(self, key: &NaiveDate) -> Self {
        TimeSeries {
            tags: self.tags,
            data: self.data.split(key).1
        }
    }

    pub fn map(self, f: fn(i64) -> i64) -> Self {
        TimeSeries {
            tags: self.tags,
            data: self.data.into_iter().map(|(k, v)| (k, f(v))).collect()
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

impl Div for TimeSeries {
    type Output = TimeSeries;

    fn div(self, rhs: Self) -> Self::Output {
        TimeSeries {
            tags: self.tags,
            data: self.data.union_with(rhs.data, |a, b| if b == 0 { 0 } else { a * 100 / b }),
        }
    }
}
