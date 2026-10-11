//! Process-wide in-flight sharing. Owners are actual IPC windows, not caller labels.
use crate::error::{AppError, AppResult};
use sha2::{Digest, Sha256};
use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
};
use tokio::{sync::watch, task::AbortHandle};

type Outcome = Result<Arc<dyn Any + Send + Sync>, (u32, String)>;
#[derive(Clone)]
pub struct WorkContext {
    cancelled: Arc<AtomicBool>,
    pub source: String,
    pub identity: String,
}
tokio::task_local! { static CONTEXT: WorkContext; }
pub fn identity(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn identity_parts(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    format!("{:x}", hash.finalize())
}
pub fn checkpoint() -> AppResult<()> {
    if CONTEXT
        .try_with(|context| context.cancelled.load(Ordering::Acquire))
        .unwrap_or(false)
    {
        Err(AppError::business(3007, "封面请求已取消"))
    } else {
        Ok(())
    }
}
pub fn trace(phase: &str, image: &str) {
    let context = CONTEXT.try_with(Clone::clone).ok();
    if let Some(path) = std::env::var_os("ISLE_ARTWORK_TRACE") {
        use std::io::Write;
        static LOCK: Mutex<()> = Mutex::new(());
        if let Ok(_guard) = LOCK.lock() {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    file,
                    "{}",
                    serde_json::json!({"at":chrono::Utc::now().timestamp_millis(),"phase":phase,"image":image,"source":context.as_ref().map(|x|x.source.as_str()).unwrap_or("native-service"),"job":context.as_ref().map(|x|x.identity.as_str())})
                );
            }
        }
    }
}
pub fn trace_bytes(phase: &str, bytes: &[u8]) {
    if std::env::var_os("ISLE_ARTWORK_TRACE").is_some() {
        trace(phase, &identity(bytes));
    }
}
fn trace_owner(phase: &str, key: &str, window: &str, slot: &str) {
    if std::env::var_os("ISLE_ARTWORK_TRACE").is_some() {
        CONTEXT.sync_scope(
            WorkContext {
                cancelled: Arc::new(AtomicBool::new(false)),
                source: format!("{window}:{slot}"),
                identity: key.into(),
            },
            || trace(phase, key),
        );
    }
}
pub async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    let context = CONTEXT.try_with(Clone::clone).ok();
    tokio::task::spawn_blocking(move || {
        let run = || {
            checkpoint()?;
            let result = work();
            checkpoint()?;
            result
        };
        if let Some(context) = context {
            CONTEXT.sync_scope(context, run)
        } else {
            run()
        }
    })
    .await
    .map_err(|error| AppError::business(3008, format!("封面工作失败：{error}")))?
}
struct Job {
    context: WorkContext,
    result: watch::Sender<Option<Outcome>>,
    abort: Mutex<Option<AbortHandle>>,
    users: Mutex<usize>,
}
struct Owner {
    revision: u64,
    job: Option<Arc<Job>>,
}
#[derive(Default)]
struct State {
    jobs: HashMap<String, Arc<Job>>,
    owners: HashMap<(String, String), Owner>,
}
#[derive(Default)]
pub struct Registry {
    state: Mutex<State>,
}
static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();
pub fn registry() -> &'static Arc<Registry> {
    REGISTRY.get_or_init(|| Arc::new(Registry::default()))
}
fn release(state: &mut State, job: Arc<Job>) {
    let mut users = job.users.lock().unwrap();
    *users -= 1;
    if *users == 0 {
        job.context.cancelled.store(true, Ordering::Release);
        if let Some(handle) = job.abort.lock().unwrap().as_ref() {
            handle.abort();
        }
        if state
            .jobs
            .get(&job.context.identity)
            .is_some_and(|current| Arc::ptr_eq(current, &job))
        {
            state.jobs.remove(&job.context.identity);
        }
        job.result
            .send_replace(Some(Err((3007, "封面请求已取消".into()))));
    }
}
impl Registry {
    pub fn cancel(&self, window: &str, slot: &str, revision: u64) {
        let mut state = self.state.lock().unwrap();
        let key = (window.to_owned(), slot.to_owned());
        if state
            .owners
            .get(&key)
            .is_some_and(|owner| owner.revision > revision)
        {
            return;
        }
        if let Some(job) = state
            .owners
            .get_mut(&key)
            .and_then(|owner| owner.job.take())
        {
            trace_owner("cancel-owner", &job.context.identity, window, slot);
            release(&mut state, job);
        }
        state.owners.insert(
            key,
            Owner {
                revision,
                job: None,
            },
        );
    }
    pub fn close_window(&self, window: &str) {
        let revision = (chrono::Utc::now().timestamp_millis().max(0) as u64) * 1000 + 999;
        for slot in ["cover", "color", "pixel"] {
            self.cancel(window, slot, revision);
        }
    }
    pub async fn run<T, F>(
        self: &Arc<Self>,
        window: &str,
        slot: &str,
        revision: u64,
        key: String,
        work: F,
    ) -> AppResult<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Future<Output = AppResult<T>> + Send + 'static,
    {
        let owner_key = (window.to_owned(), slot.to_owned());
        let mut work = Some(work);
        let job = {
            let mut state = self.state.lock().unwrap();
            if state
                .owners
                .get(&owner_key)
                .is_some_and(|owner| owner.revision >= revision)
            {
                return Err(AppError::business(3007, "过期封面请求"));
            }
            // Join before releasing the previous lease so unchanged work survives replacement.
            let job = if let Some(job) = state.jobs.get(&key) {
                trace_owner("join", &key, window, slot);
                job.clone()
            } else {
                if state.jobs.len() >= 8 {
                    return Err(AppError::business(3005, "封面在途任务已满"));
                }
                let (result, _) = watch::channel(None);
                let job = Arc::new(Job {
                    context: WorkContext {
                        cancelled: Arc::new(AtomicBool::new(false)),
                        source: if window == "native-download" {
                            CONTEXT
                                .try_with(|context| format!("{}/download", context.source))
                                .unwrap_or_else(|_| format!("{window}:{slot}"))
                        } else {
                            format!("{window}:{slot}")
                        },
                        identity: key.clone(),
                    },
                    result,
                    abort: Mutex::new(None),
                    users: Mutex::new(0),
                });
                let owned = job.clone();
                let registry = self.clone();
                let work = work.take().unwrap();
                let handle = tokio::spawn(CONTEXT.scope(job.context.clone(), async move {
                    let _completion = Completion {
                        registry: registry.clone(),
                        job: owned.clone(),
                    };
                    trace("start", &owned.context.identity);
                    let outcome = work
                        .await
                        .and_then(|value| {
                            checkpoint()?;
                            Ok(value)
                        })
                        .map(|value| Arc::new(value) as Arc<dyn Any + Send + Sync>)
                        .map_err(|error| (error.code(), error.to_string()));
                    let mut state = registry.state.lock().unwrap();
                    if state
                        .jobs
                        .get(&owned.context.identity)
                        .is_some_and(|current| Arc::ptr_eq(current, &owned))
                    {
                        state.jobs.remove(&owned.context.identity);
                    }
                    trace(
                        if outcome.is_ok() {
                            "complete"
                        } else {
                            "failed"
                        },
                        &owned.context.identity,
                    );
                    owned.result.send_replace(Some(outcome));
                }));
                *job.abort.lock().unwrap() = Some(handle.abort_handle());
                state.jobs.insert(key, job.clone());
                job
            };
            *job.users.lock().unwrap() += 1;
            if let Some(old) = state
                .owners
                .get_mut(&owner_key)
                .and_then(|owner| owner.job.take())
            {
                release(&mut state, old);
            }
            state.owners.insert(
                owner_key.clone(),
                Owner {
                    revision,
                    job: Some(job.clone()),
                },
            );
            job
        };
        drop(work); // A joiner does not retain a duplicate input buffer/future.
        let _lease = Lease {
            registry: self.clone(),
            owner: owner_key,
            revision,
        };
        let mut result = job.result.subscribe();
        loop {
            if let Some(outcome) = result.borrow_and_update().clone() {
                if !self
                    .state
                    .lock()
                    .unwrap()
                    .owners
                    .get(&_lease.owner)
                    .is_some_and(|owner| {
                        owner.revision == revision
                            && owner
                                .job
                                .as_ref()
                                .is_some_and(|current| Arc::ptr_eq(current, &job))
                    })
                {
                    return Err(AppError::business(3007, "过期封面结果"));
                }
                return match outcome {
                    Ok(value) => value
                        .downcast_ref::<T>()
                        .cloned()
                        .ok_or_else(|| AppError::business(3008, "封面结果类型不匹配")),
                    Err((code, message)) => Err(AppError::business(code, message)),
                };
            }
            result
                .changed()
                .await
                .map_err(|_| AppError::business(3007, "封面任务结束"))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    #[tokio::test]
    async fn shared_work_survives_one_window_and_stops_after_last_owner() {
        let registry = Arc::new(Registry::default());
        let count = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(tokio::sync::Notify::new());
        let a = {
            let r = registry.clone();
            let c = count.clone();
            let g = gate.clone();
            tokio::spawn(async move {
                r.run("main", "cover", 1, "same-v1".into(), async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    g.notified().await;
                    Ok(42u32)
                })
                .await
            })
        };
        while count.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
        let b = {
            let r = registry.clone();
            tokio::spawn(async move {
                r.run("floating", "cover", 1, "same-v1".into(), async {
                    panic!("duplicate operation started");
                    #[allow(unreachable_code)]
                    Ok(0u32)
                })
                .await
            })
        };
        while registry.state.lock().unwrap().owners.len() < 2 {
            tokio::task::yield_now().await;
        }
        registry.cancel("main", "cover", 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        gate.notify_one();
        assert!(a.await.unwrap().is_err());
        assert_eq!(b.await.unwrap().unwrap(), 42);
        assert!(registry.state.lock().unwrap().jobs.is_empty());
    }
    #[tokio::test]
    async fn cancellation_before_request_and_old_cancel_cannot_resurrect_or_cancel_new_version() {
        let r = Arc::new(Registry::default());
        r.cancel("float", "cover", 5);
        assert!(r
            .run("float", "cover", 5, "v1".into(), async { Ok(1u32) })
            .await
            .is_err());
        let gate = Arc::new(tokio::sync::Notify::new());
        let task = {
            let r = r.clone();
            let g = gate.clone();
            tokio::spawn(async move {
                r.run("float", "cover", 6, "v2".into(), async move {
                    g.notified().await;
                    Ok(2u32)
                })
                .await
            })
        };
        while r.state.lock().unwrap().jobs.is_empty() {
            tokio::task::yield_now().await;
        }
        r.cancel("float", "cover", 5);
        assert_eq!(r.state.lock().unwrap().jobs.len(), 1);
        gate.notify_one();
        assert_eq!(task.await.unwrap().unwrap(), 2);
    }
    #[tokio::test]
    async fn final_cancel_drops_async_work_and_failure_is_retryable() {
        struct Dropped(Arc<AtomicBool>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let r = Arc::new(Registry::default());
        let dropped = Arc::new(AtomicBool::new(false));
        let started = Arc::new(AtomicBool::new(false));
        let task = {
            let r = r.clone();
            let d = dropped.clone();
            let s = started.clone();
            tokio::spawn(async move {
                r.run("main", "cover", 1, "image".into(), async move {
                    let _drop = Dropped(d);
                    s.store(true, Ordering::SeqCst);
                    std::future::pending::<()>().await;
                    Ok(1u32)
                })
                .await
            })
        };
        while !started.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
        r.cancel("main", "cover", 1);
        assert!(task.await.unwrap().is_err());
        while !dropped.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
        assert!(r
            .run::<u32, _>("main", "cover", 2, "image".into(), async {
                Err(AppError::network("fixture failure"))
            })
            .await
            .is_err());
        assert_eq!(
            r.run("main", "cover", 3, "image".into(), async { Ok(3u32) })
                .await
                .unwrap(),
            3
        );
    }
}
struct Lease {
    registry: Arc<Registry>,
    owner: (String, String),
    revision: u64,
}
struct Completion {
    registry: Arc<Registry>,
    job: Arc<Job>,
}
impl Drop for Completion {
    fn drop(&mut self) {
        if let Ok(mut state) = self.registry.state.lock() {
            if state
                .jobs
                .get(&self.job.context.identity)
                .is_some_and(|current| Arc::ptr_eq(current, &self.job))
            {
                state.jobs.remove(&self.job.context.identity);
            }
        }
        let missing = self.job.result.borrow().is_none();
        if missing {
            self.job
                .result
                .send_replace(Some(Err((3008, "封面任务异常结束".into()))));
        }
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut state = self.registry.state.lock().unwrap();
        if let Some(owner) = state.owners.get_mut(&self.owner) {
            if owner.revision == self.revision {
                if let Some(job) = owner.job.take() {
                    release(&mut state, job);
                }
            }
        }
        if self.owner.0 == "native-download" {
            state.owners.remove(&self.owner);
        }
    }
}
