use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use run_task::prelude::*;

struct TimeSeries {
    time_series: BTreeMap<u64, i64>,
}

impl Default for TimeSeries {
    fn default() -> Self {
        let mut data = BTreeMap::new();
        data.insert(1, 42);
        data.insert(2, 1024);
        TimeSeries { time_series: data }
    }
}

struct OHLCA {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl std::fmt::Debug for OHLCA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OHLCA {{ open: {}, high: {}, low: {}, close: {}, volume: {} }}",
            self.open, self.high, self.low, self.close, self.volume
        )
    }
}

#[derive(Clone)]
struct TestTaskA;

impl Runnable<TimeSeries, OHLCA> for TestTaskA {
    fn name(&self) -> String {
        "TestTask_A".to_string()
    }

    fn run(&self, data: &TimeSeries, _at: u64) -> Result<OHLCA, TaskError<OHLCA>> {
        let a = data.time_series.get(&1).unwrap();
        let b = data.time_series.get(&2).unwrap();

        Ok(OHLCA {
            open: *a as f64,
            high: *b as f64,
            low: *a as f64,
            close: *b as f64,
            volume: 1000.0,
        })
    }
}

struct TestTaskB;

impl Runnable<TimeSeries, OHLCA> for TestTaskB {
    fn name(&self) -> String {
        "TestTask_B".to_string()
    }

    fn run(&self, data: &TimeSeries, _at: u64) -> Result<OHLCA, TaskError<OHLCA>> {
        let a = data.time_series.get(&1).unwrap();
        let b = data.time_series.get(&2).unwrap();

        Ok(OHLCA {
            open: (*a * 100) as f64,
            high: (*b * 100) as f64,
            low: (*a * 100) as f64,
            close: (*b * 100) as f64,
            volume: 1024.0,
        })
    }
}

#[tokio::main]
async fn main() {
    let data = Arc::new(RwLock::new(TimeSeries::default()));
    let (ctx, mut receiver) = ContextBuilder::new()
        .with_task(TestTaskA)
        .with_task(TestTaskB)
        .with_data(data)
        .with_interval(TaskInterval::Seconds(3))
        .build();

    spawn_runner(ctx);

    loop {
        let data = receiver.recv().await.unwrap();
        println!("{:?}", data);
    }
}
