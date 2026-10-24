//! Bounded executor for synchronous native APIs and CPU-heavy helpers.

use std::sync::{Arc, LazyLock};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const MAX_NATIVE_TASKS: usize = 4;
const MAX_BACKGROUND_TASKS: usize = 2;

struct NativeTaskPool {
    slots: Arc<Semaphore>,
}

impl NativeTaskPool {
    fn new(capacity: usize) -> Self {
        Self {
            slots: Arc::new(Semaphore::new(capacity)),
        }
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, NativeTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let permit: OwnedSemaphorePermit = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| NativeTaskError::Saturated)?;

        tauri::async_runtime::spawn_blocking(move || {
            let _permit = permit;
            operation()
        })
        .await
        .map_err(|error| NativeTaskError::Worker(error.to_string()))
    }
}

static NATIVE_TASKS: LazyLock<NativeTaskPool> =
    LazyLock::new(|| NativeTaskPool::new(MAX_NATIVE_TASKS));
static BACKGROUND_TASKS: LazyLock<NativeTaskPool> =
    LazyLock::new(|| NativeTaskPool::new(MAX_BACKGROUND_TASKS));

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NativeTaskError {
    Saturated,
    Worker(String),
}

impl std::fmt::Display for NativeTaskError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Saturated => formatter.write_str("native worker capacity is full"),
            Self::Worker(error) => write!(formatter, "native worker failed: {error}"),
        }
    }
}

pub(crate) async fn run_native<T, F>(operation: F) -> Result<T, NativeTaskError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    NATIVE_TASKS.run(operation).await
}

pub(crate) async fn run_background<T, F>(operation: F) -> Result<T, NativeTaskError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    BACKGROUND_TASKS.run(operation).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_work_when_native_capacity_is_full() {
        let pool = NativeTaskPool::new(1);
        let permit = pool.slots.clone().try_acquire_owned().unwrap();
        assert_eq!(pool.run(|| ()).await, Err(NativeTaskError::Saturated));
        drop(permit);
        assert_eq!(pool.run(|| 42).await, Ok(42));
    }

    #[tokio::test]
    async fn runs_work_off_the_async_caller_and_releases_its_slot() {
        let before = std::thread::current().id();
        let pool = NativeTaskPool::new(1);
        let worker = pool.run(|| std::thread::current().id()).await.unwrap();
        assert_ne!(before, worker);
        assert_eq!(pool.slots.available_permits(), 1);
    }
}
