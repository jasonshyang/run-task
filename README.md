# Run Task
This is a light weight simple tool to run concurrent data tasks and send consolidated output back via a tokio::mpsc channel.

The tasks will run periodically as specified by the `TaskInterval` provided in the `Context`, and the underlying data needs to be guarded by a `RwLock` so that it can be updated as the task runner runs (by acquiring the write lock).

## Usage

### How To Use
Import prelude
```rust
use run_task::prelude::*;
```

Implement `Runnable<T, D>` for a data task you want to run that deals with type `T` and output type `D`, you can define multiple tasks as long as they are all dealing with the same type `T` and `D`.
```rust
impl Runnable<YourInputDataType, YourOutputDataType> for TestTaskA {
    fn name(&self) -> String {
        "Your Task Name".to_string()
    }

    fn run(&self, data: &YourInputDataType, start: u64, end: u64) -> Result<YourOutputDataType, TaskError<YourOutputDataType>> {
        // you can implement your actual task here
        // you can access the start and end of the time interval
        Ok(YourOutputDataType)
    }
}
```

Build the Task Runner `Context` with `ContextBuilder`:
- You can add your task with `.with_task()`, or `.with_tasks()` to add a vector of tasks.
- You can add data with `.with_data()`. The underlying data needs to be wrapped with `Arc<RwLock<>>` so you can write to it when the runner runs. Because the `ContextBuilder` requires your input data struct to implement `Default`, you can skip the `.with_data()`, and a default instance of your struct will be created and wrapped in `Arc<RwLock<>>`
- You can add config with `.with_config()`. The Runner has a default config, but you can overwrite that with your own `RunnerConfig`, and add that to the `ContextBuilder` by calling `.with_config()`.
- You can add the `TaskInterval` with `.with_interval()`, this can be in `Micros`, `Millis`, `Seconds`, or `Minutes`, you should align that with your input data struct if you have a time data there. The `Runner` will output the data in the same format (e.g. millis or micros) based on this setting

At the end, you need to call `.build()` to create a `Context` for the `Runner`. You will get back a `BuildResult` which is a tuple containing:
- A `Context` for you to use to call the `Runner.run()`.
- A `mpsc::Receiver`, you use this to get the output data by calling `.recv()`.
- A shared reference to the underlying data, if you built the context with your own data, you can discard this.

```rust
let runner_config = RunnerConfig::new(1024, 16, std::time::Duration::from_secs(5));
let (ctx, mut receiver, data) = ContextBuilder::new()
    .with_task(TestTaskA)
    .with_task(TestTaskB)
    .with_interval(TaskInterval::Seconds(2))
    .with_config(runner_config)
    .build();

```
Finally, you can create a new instance of `Runner` and run with the `Context`. 
```rust
let runner = Runner::new(ctx);
let runner_handle = tokio::spawn(async move {
    runner.run().await.unwrap();
});
```

You get the data back by calling `receiver.recv()`.
```rust
let receiver_handle = tokio::spawn(async move {
    while let Some(data) = receiver.recv().await {
        println!("Timestamp: {}", chrono::Utc::now());
        println!("Data: {:?}", data);
        println!("---");
    }
});
```

### Example
To see what the output looks like you can try:
```zsh
cd run-task
cargo run --example timeseries
```