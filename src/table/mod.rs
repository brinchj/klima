use chrono;
use chrono::NaiveDate;
use im;
use std::collections::BTreeSet;
use std::ops::Add;

#[derive(Default, Clone)]
pub struct TimeSeries {
    pub tags: im::OrdSet<String>,
    pub data: im::OrdMap<NaiveDate, i64>,
}

impl TimeSeries {
    pub fn new(tags: im::OrdSet<String>, data: im::OrdMap<NaiveDate, i64>) -> TimeSeries {
        TimeSeries {
            tags,
            data
        }
    }

    pub fn unit(tags: im::OrdSet<String>, date: NaiveDate, value: i64) -> TimeSeries {
        TimeSeries {
            tags,
            data: im::OrdMap::unit(date, value),
        }
    }

    pub fn accumulative(&self) -> Self {
        let init = (0i64, im::OrdMap::new());
        let (sum, data) = self.data.iter().fold(init, |(sum, new), (t, y)| {
            ((*y + sum), new.update(*t, *y + sum))
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
