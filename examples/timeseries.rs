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
        data.insert(2, 1);
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
        let (_, latest) = data.time_series.iter().last().unwrap();

        Ok(OHLCA {
            open: (*latest * 100) as f64,
            high: (*latest * 100) as f64,
            low: (*latest * 100) as f64,
            close: (*latest * 100) as f64,
            volume: 1024.0,
        })
    }
}

#[tokio::main]
async fn main() {
    let data = Arc::new(RwLock::new(TimeSeries::default()));
    let data_clone = Arc::clone(&data);
    let (ctx, mut receiver) = ContextBuilder::new()
        .with_task(TestTaskA)
        .with_task(TestTaskB)
        .with_data(data_clone)
        .with_interval(TaskInterval::Seconds(3))
        .build();

    spawn_runner(ctx);

    let handle1 = tokio::spawn(async move {
        loop {
            let data = receiver.recv().await.unwrap();
            println!("{:?}", data);
        }
    });

    // spawn tasks to write to the data
    let handle2 = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("Writing to data, expect to see TestTask_B result changes");
        let mut data = data.write().await;
        data.time_series.insert(3, 100);
    });

    let _ = tokio::join!(handle1, handle2);
    
}
