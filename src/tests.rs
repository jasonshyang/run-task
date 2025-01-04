#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::Duration;
    use crate::prelude::*;

    #[derive(Default)]
    struct TestData {
        value: i32,
    }

    #[derive(Debug, PartialEq)]
    struct TestResult {
        value: i32,
    }

    #[derive(Clone)]
    struct TestTask {
        multiplier: i32,
    }

    impl Runnable<TestData, TestResult> for TestTask {
        fn name(&self) -> String {
            format!("TestTask_{}", self.multiplier)
        }

        fn run(&self, data: &TestData, _start: u64, _end: u64) -> Result<TestResult, TaskError<TestResult>> {
            Ok(TestResult {
                value: data.value * self.multiplier
            })
        }
    }

    #[tokio::test]
    async fn test_single_task() {
        let data = Arc::new(RwLock::new(TestData { value: 42 }));
        let task = TestTask { multiplier: 2 };
        
        let (ctx, mut receiver, _) = ContextBuilder::new()
            .with_task(task)
            .with_data(data.clone())
            .with_interval(TaskInterval::Millis(100))
            .build();

        let runner = crate::Runner::new(ctx);
        
        tokio::spawn(async move {
            runner.run().await
        });

        let result = tokio::time::timeout(
            Duration::from_millis(200),
            receiver.recv()
        ).await.unwrap().unwrap();

        assert_eq!(result.get("TestTask_2").unwrap().value, 84);
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let data = Arc::new(RwLock::new(TestData { value: 10 }));
        let task1 = TestTask { multiplier: 2 };
        let task2 = TestTask { multiplier: 3 };
        
        let (ctx, mut receiver, _) = ContextBuilder::new()
            .with_task(task1)
            .with_task(task2)
            .with_data(data.clone())
            .with_interval(TaskInterval::Millis(100))
            .build();

        let runner = crate::Runner::new(ctx);

        tokio::spawn(async move {
            runner.run().await
        });

        let result = tokio::time::timeout(
            Duration::from_millis(200),
            receiver.recv()
        ).await.unwrap().unwrap();

        assert_eq!(result.get("TestTask_2").unwrap().value, 20);
        assert_eq!(result.get("TestTask_3").unwrap().value, 30);
    }

    #[tokio::test]
    async fn test_data_update() {
        let data = Arc::new(RwLock::new(TestData { value: 10 }));
        let task = TestTask { multiplier: 2 };
        
        let (ctx, mut receiver, _) = ContextBuilder::new()
            .with_task(task)
            .with_data(data.clone())
            .with_interval(TaskInterval::Millis(100))
            .build();

        let runner = crate::Runner::new(ctx);
        
        tokio::spawn(async move {
            runner.run().await
        });

        let result1 = tokio::time::timeout(
            Duration::from_millis(200),
            receiver.recv()
        ).await.unwrap().unwrap();

        assert_eq!(result1.get("TestTask_2").unwrap().value, 20);

        {
            let mut data_guard = data.write().await;
            data_guard.value = 20;
        }

        let result2 = tokio::time::timeout(
            Duration::from_millis(200),
            receiver.recv()
        ).await.unwrap().unwrap();

        assert_eq!(result2.get("TestTask_2").unwrap().value, 40);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let data = Arc::new(RwLock::new(TestData { value: 10 }));
        let task = TestTask { multiplier: 2 };
        
        let (ctx, mut receiver, _) = ContextBuilder::new()
            .with_task(task)
            .with_data(data.clone())
            .with_interval(TaskInterval::Millis(100))
            .build();

        let runner = crate::Runner::new(ctx);
        let runner_handle = tokio::spawn(async move {
            runner.run().await
        });

        let _ = tokio::time::timeout(
            Duration::from_millis(200),
            receiver.recv()
        ).await.unwrap().unwrap();

        runner_handle.abort();

        assert!(runner_handle.await.unwrap_err().is_cancelled());
    }
}