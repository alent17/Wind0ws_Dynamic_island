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

// Transport must not wait behind metadata, artwork, or device enumeration.
// Keep clicks ordered and admission bounded, including when an IPC is cancelled.
struct TransportTaskPool {
    slots: Arc<Semaphore>,
    serial: Arc<Semaphore>,
}

impl TransportTaskPool {
    fn new() -> Self {
        Self {
            slots: Arc::new(Semaphore::new(8)),
            serial: Arc::new(Semaphore::new(1)),
        }
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, NativeTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let admission = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| NativeTaskError::Saturated)?;
        let serial = self
            .serial
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| NativeTaskError::Saturated)?;
        tauri::async_runtime::spawn_blocking(move || {
            let _admission = admission;
            let _serial = serial;
            operation()
        })
        .await
        .map_err(|error| NativeTaskError::Worker(error.to_string()))
    }
}

static TRANSPORT_TASKS: LazyLock<TransportTaskPool> = LazyLock::new(TransportTaskPool::new);

pub(crate) async fn run_transport<T, F>(operation: F) -> Result<T, NativeTaskError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    TRANSPORT_TASKS.run(operation).await
}

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
    async fn transport_is_independent_of_saturated_native_and_background_work() {
        let _native = NATIVE_TASKS
            .slots
            .clone()
            .try_acquire_many_owned(MAX_NATIVE_TASKS as u32)
            .unwrap();
        let _background = BACKGROUND_TASKS
            .slots
            .clone()
            .try_acquire_many_owned(MAX_BACKGROUND_TASKS as u32)
            .unwrap();
        assert_eq!(TransportTaskPool::new().run(|| 42).await, Ok(42));
    }

    #[tokio::test]
    async fn transport_is_bounded_and_retains_serial_ownership_after_cancellation() {
        let pool = Arc::new(TransportTaskPool::new());
        let all = pool.slots.clone().try_acquire_many_owned(8).unwrap();
        assert_eq!(pool.run(|| ()).await, Err(NativeTaskError::Saturated));
        drop(all);
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let first_pool = pool.clone();
        let first = tokio::spawn(async move {
            first_pool
                .run(move || {
                    let _ = started.send(());
                    wait.recv_timeout(std::time::Duration::from_secs(3))
                        .unwrap();
                })
                .await
        });
        ready.await.unwrap();
        first.abort();
        let second_pool = pool.clone();
        let (second_started, mut second_ready) = tokio::sync::oneshot::channel();
        let second = tokio::spawn(async move {
            second_pool
                .run(move || {
                    let _ = second_started.send(());
                })
                .await
        });
        for _ in 0..100 {
            if pool.slots.available_permits() == 6 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(pool.serial.available_permits(), 0);
        assert!(matches!(
            second_ready.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        release.send(()).unwrap();
        assert_eq!(second.await.unwrap(), Ok(()));
        assert_eq!(pool.slots.available_permits(), 8);
    }

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
