# Run Task
This is a light weight simple tool to run concurrent data tasks and send consolidated output back via a tokio::mpsc channel.

The tasks will run periodically as specified by the `TaskInterval` provided in the `Context`, and the underlying data needs to be guarded by a `RwLock` so that it can be updated as the task runner runs (by acquiring the write lock).

## Usage

### How To Use
Import prelude
```rust
use run_task::prelude::*;
```

Implement `Runnable<T>` for a data task you want to run that deals with type `T`, you can define multiple tasks as long as they are all dealing with the same type `T`.
```rust
impl Runnable<T> for YourTask {
    fn name(&self) -> String {
        "Your Task Name".to_string()
    }

    fn run(&self, data: &DB, _at: u64) -> Result<Data, TaskError> {
        // define the data task here, e.g. calculating OHLCV (Open, High, Low, Close, Volume) for a trade time series data
        // wrap the result in Value type, this can be f64, i64, bool or String.
        // wrap the value in a Data type which serves as a container, it can be a Scalar, Vec, HashMap, or None if no value
    }
} 
```

Build the Task Runner Context with `ContextBuilder`, you can add your task by calling `.with_task()`, or call `.with_tasks()` to add a vector of tasks, add the underlying data wrapped with `Arc<RwLock<>>` so you can update the data as the task runner does its job.

Creating a `Context` will give you back a `mpsc::Receiver`, you use this to get the output data.

```rust
let data = Arc::new(RwLock::new(DB::default()));
let (ctx, mut receiver) = ContextBuilder::new()
    .with_task(TestTaskA)
    .with_task(TestTaskB)
    .with_data(data)
    .with_interval(TaskInterval::Seconds(3))
    .build();
```

Finally, you can use the `spawn_runner()` function to run the tasks (this will run the task runner inside a `tokio::spawn`). You get the data back by calling `receiver.recv()`.
```rust
spawn_runner(ctx);
```

### Example
To see what the output looks like you can try:
```zsh
cd run-task
cargo run --example custom_struct
```