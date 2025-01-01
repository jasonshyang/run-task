use std::sync::Arc;
use tokio::sync::RwLock;

use run_task::prelude::*;

#[derive(Clone)]
struct TestTask;

impl Runnable<i64> for TestTask {
    fn name(&self) -> String {
        "TestTask".to_string()
    }

    fn run(&self, data: &i64, _at: u64) -> Result<Data, TaskError> {
        let result = data + 1;
        Ok(Data::Scalar(Value::Int(result)))
    }
}

#[tokio::main]
async fn main() {
    let data = Arc::new(RwLock::new(42));
    let (ctx, mut receiver) = ContextBuilder::new()
        .with_task(TestTask)
        .with_data(data)
        .with_interval(TaskInterval::Seconds(3))
        .build();

    spawn_runner(ctx);

    loop {
        let data = receiver.recv().await.unwrap();
        println!("{:?}", data);
    }
}
