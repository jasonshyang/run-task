use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::signal;

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
    from: u64,
    to: u64,
}

impl std::fmt::Debug for OHLCA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OHLCA {{ open: {}, high: {}, low: {}, close: {}, volume: {}, from: {}, to: {} }}",
            self.open, self.high, self.low, self.close, self.volume, self.from, self.to
        )
    }
}

#[derive(Clone)]
struct TestTaskA;
#[derive(Clone)]
struct TestTaskB;

impl Runnable<TimeSeries, OHLCA> for TestTaskA {
    fn name(&self) -> String {
        "TestTask_A".to_string()
    }

    fn run(&self, data: &TimeSeries, start: u64, end: u64) -> Result<OHLCA, TaskError<OHLCA>> {
        let values: Vec<_> = data.time_series.values().collect();
        Ok(OHLCA {
            open: *values[0] as f64,
            high: **values.iter().max().unwrap() as f64,
            low: **values.iter().max().unwrap() as f64,
            close: *values[values.len() - 1] as f64,
            volume: values.len() as f64,
            from: start,
            to: end,
        })
    }
}

impl Runnable<TimeSeries, OHLCA> for TestTaskB {
    fn name(&self) -> String {
        "TestTask_B".to_string()
    }

    fn run(&self, data: &TimeSeries, start: u64, end: u64) -> Result<OHLCA, TaskError<OHLCA>> {
        let values: Vec<_> = data.time_series.values().collect();
        Ok(OHLCA {
            open: *values[0] as f64 * 100.0,
            high: **values.iter().max().unwrap() as f64 * 100.0,
            low: **values.iter().max().unwrap() as f64 * 100.0,
            close: *values[values.len() - 1] as f64 * 100.0,
            volume: values.len() as f64 * 100.0,
            from: start,
            to: end,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = Arc::new(RwLock::new(TimeSeries::default()));
    let data_clone = Arc::clone(&data);
    let runner_config = RunnerConfig::new(1024, 16, std::time::Duration::from_secs(5));
    let (ctx, mut receiver) = ContextBuilder::new()
        .with_config(runner_config)
        .with_task(TestTaskA)
        .with_task(TestTaskB)
        .with_data(data_clone)
        .with_interval(TaskInterval::Seconds(2))
        .build();

    let runner = Runner::new(ctx);
    let runner_handle = tokio::spawn(async move {
        if let Err(e) = runner.run().await {
            eprintln!("Runner error: {:?}", e);
        }
    });

    let receiver_handle = tokio::spawn(async move {
        while let Some(data) = receiver.recv().await {
            println!("Timestamp: {}", chrono::Utc::now());
            println!("Data: {:?}", data);
            println!("---");
        }
    });

    let writer_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            println!("Writing to data...");
            let mut data = data.write().await;
            let next_key = data.time_series.keys().max().unwrap_or(&0) + 1;
            data.time_series.insert(next_key, 42);
        }
    });

    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Shutting down...");
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
            println!("Runtime completed...");
        }
    }

    runner_handle.abort();
    receiver_handle.abort();
    writer_handle.abort();

    Ok(())
}
