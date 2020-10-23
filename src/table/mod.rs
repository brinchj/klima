use chrono;
use chrono::{Datelike, NaiveDate};
use im;
use std::collections::BTreeSet;
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

    pub fn len(&self) -> usize {
        self.series.len()
    }

    pub fn future_goal(self, date: NaiveDate, goal: i64) -> Self {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        let final_date = self.series.iter().map(last_date).max().unwrap();

        let datapoint = |ts: &TimeSeries| *ts.data.iter().last().unwrap().1;
        let final_sum: i64 = self.series.iter().map(datapoint).sum();

        let mut running_date = final_date;
        let all_days = (date - running_date).num_days();

        let mut goal_data = im::OrdMap::new();
        while running_date < date {
            running_date = (running_date + chrono::Duration::days(31))
                .with_day(1)
                .unwrap();

            let days_spent = (running_date - final_date).num_days();
            let progress = ((goal - final_sum) * days_spent) / all_days;
            goal_data.insert(running_date, final_sum + progress);
        }

        let tags = im::OrdSet::unit("Mål, Total".to_string());
        let mut series = self.series;
        series.push(TimeSeries::new(tags, goal_data));

        TimeSeriesGroup { series }
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

    pub fn accumulative(self) -> Self {
        let init = (0i64, im::OrdMap::new());
        let (sum, data) = self.data.into_iter().fold(init, |(sum, new), (t, y)| {
            ((y + sum), new.update(t, y + sum))
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
            data: self.data.union(rhs.data),
        }
    }
}
