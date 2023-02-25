use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use flume::{Receiver, Sender};
use futures::{
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};
use sqlx::SqlitePool;
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::{debug, error};

use crate::{bus::GlobalBus, device::Sources};

type TaskFn<F> = fn(task: Task) -> F;
type TaskFnArg<A, F> = fn(argument: A, task: Task) -> F;

#[derive(Clone)]
pub struct Task {
    running: Arc<dashmap::DashMap<String, oneshot::Sender<()>>>,
    tx: Sender<TaskHandle>,

    // Data all tasks share and cooperate on
    pub db: Arc<SqlitePool>,
    pub sources: Arc<Sources>,
    pub bus: Arc<GlobalBus>,
}

impl Task {
    /// Spawn a named task, if there is a task already running with the same name it will be killed beforehand
    pub fn spawn<L, F>(&self, label: L, callback: TaskFn<F>)
    where
        F: Future<Output = Result<()>> + Send + 'static,
        L: Into<String>,
    {
        let label = label.into();
        let (tx, rx) = oneshot::channel();

        if let Some(old) = self.running.insert(label.clone(), tx) {
            old.send(()).expect("Close previous task");
        }

        let task = self.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = rx => Ok(()),
                result = callback(task) => result
            }
        });

        self.tx
            .send(TaskHandle { label, handle })
            .expect("Could not tell task group about join handle");
    }

    /// Spawn a named task, if there is a task already running with the same name it will be killed beforehand
    /// Only difference to spawn is the ability to pass some data
    pub fn spawn_with_argument<L, A, F>(&self, label: L, argument: A, callback: TaskFnArg<A, F>)
    where
        F: Future<Output = Result<()>> + Send + 'static,
        L: Into<String>,
        A: Send + 'static,
    {
        let label = label.into();
        let (tx, rx) = oneshot::channel();

        if let Some(old) = self.running.insert(label.clone(), tx) {
            old.send(()).expect("Close previous task");
        }

        let task = self.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = rx => Ok(()),
                result = callback(argument, task) => result
            }
        });

        self.tx
            .send(TaskHandle { label, handle })
            .expect("Could not tell task group about join handle");
    }

    pub fn has_task(&self, name: &str) -> bool {
        self.running.contains_key(name)
    }
}

pub fn create_group<F>(
    callback: TaskFn<F>,
    db: SqlitePool,
    sources: Sources,
    bus: GlobalBus,
) -> Group
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let (etx, erx) = oneshot::channel();
    let running = Arc::new(dashmap::DashMap::new());

    let db = Arc::new(db);
    let sources = Arc::new(sources);
    let bus = Arc::new(bus);

    running.insert("init".into(), etx);

    let (tx, rx) = flume::unbounded();

    let handle = TaskHandle {
        label: "init".into(),

        handle: tokio::spawn(async move {
            tokio::select! {
                _ = erx => Ok(()),
                result = callback(Task { running, tx, db, sources, bus }) => result
            }
        }),
    };

    Group {
        rx,
        track: vec![handle].into_iter().collect(),
    }
}

/**
 * A group of tasks, only one task per unique task label can be running,
 * The initial task gives you the oportunity to spawn more tasks into the same group
 */
pub struct Group {
    track: FuturesUnordered<TaskHandle>,
    rx: Receiver<TaskHandle>,
}

impl Group {
    pub async fn complete(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some((id, result)) = self.track.next() => {
                    match result {
                        Ok(_) => debug!("task exit {id}"),
                        Err(e) => error!("task failed {id} with error {e:?}"),
                    }
                },
                Ok(add) =  self.rx.recv_async() => {
                    debug!("task spawned {}", add.label);
                    self.track.push(add);
                }
                else => {
                    return Ok(());
                }
            }
        }
    }
}

pub struct TaskHandle {
    pub label: String,
    handle: JoinHandle<Result<()>>,
}

impl Future for TaskHandle {
    type Output = (String, Result<()>);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let label = self.label.clone();

        self.handle.poll_unpin(cx).map(|res| {
            let out = res.map_err(|join| join.into()).and_then(|inner| inner);
            (label, out)
        })
    }
}
