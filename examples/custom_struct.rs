use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use run_task::prelude::*;

struct DB {
    data: BTreeMap<String, i64>,
}

impl Default for DB {
    fn default() -> Self {
        let mut data = BTreeMap::new();
        data.insert("test_key_a".to_string(), 42);
        data.insert("test_key_b".to_string(), 1024);
        DB { data }
    }
}

#[derive(Clone)]
struct TestTaskA;

impl Runnable<DB> for TestTaskA {
    fn name(&self) -> String {
        "TestTask_A".to_string()
    }

    fn run(&self, data: &DB, _at: u64) -> Result<Data, TaskError> {
        let result = data.data.get("test_key_a").unwrap() + 1;

        Ok(Data::Scalar(Value::Int(result)))
    }
}

struct TestTaskB;

impl Runnable<DB> for TestTaskB {
    fn name(&self) -> String {
        "TestTask_B".to_string()
    }

    fn run(&self, data: &DB, _at: u64) -> Result<Data, TaskError> {
        let result = data.data.get("test_key_b").unwrap() + 100;

        Ok(Data::Scalar(Value::Int(result)))
    }
}

#[tokio::main]
async fn main() {
    let data = Arc::new(RwLock::new(DB::default()));
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
