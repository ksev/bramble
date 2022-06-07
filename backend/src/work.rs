use std::any::Any;

use anyhow::Result;
use dashmap::DashMap;
use futures::Future;
use once_cell::sync::Lazy;
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot::{channel, Receiver, Sender},
    },
    task::JoinHandle,
};

static TASK_HANDLES: Lazy<DashMap<String, tokio::sync::OnceCell<TaskHandle>>> =
    Lazy::new(|| DashMap::new());

/// A handle to manage long running async tasks
pub struct TaskHandle {
    task_handle: JoinHandle<Result<()>>,
    // This should always be a unbounded sender
    communication: Box<dyn Any + Send + Sync>,
    close_shot: Sender<()>,
}

impl TaskHandle {
    pub fn new<T>(
        task_handle: JoinHandle<Result<()>>,
        close_shot: Sender<()>,
        communication: UnboundedSender<T>,
    ) -> TaskHandle
    where
        T: Send + Sync + 'static,
    {
        TaskHandle {
            task_handle,
            close_shot,
            communication: Box::new(communication),
        }
    }

    /// Get a reified channel from the task handle
    pub fn communications_channel<T>(&self) -> UnboundedSender<T> where T: 'static {
        self.communication.downcast_ref::<UnboundedSender<T>>().cloned().expect("communications channel downcast failed, this is a bug")
    }
}

/// Check against the live list of a job has been registered
pub fn has_job<N>(name: N) -> bool
where
    N: AsRef<str>,
{
    TASK_HANDLES.contains_key(name.as_ref())
}

/// Start a new job, this function will only allow unique jobs, so if a job exists 
/// you will simply get handle if not the init function is called, 
/// this will also protected against the thundering herd problem and make sure only one job gets initialized
pub async fn start_job<N, C, F, M>(name: N, init: C) -> Result<TaskHandle>
where
    N: Into<String>,
    C: Fn(UnboundedReceiver<M>, Receiver<()>) -> F,
    F: Future<Output = Result<JoinHandle<Result<()>>>>,
    M: Send + 'static,
{
    // Setup barrier

    let (sender, receiver) = unbounded_channel();
    let (close_sender, close_recv) = channel();

    let task_handle = init(receiver, close_recv).await?;
    /* *

    if let Some(handle) = TASK_HANDLES.insert(name.into(), tokio::sync::Barrier) {
        handle.close_shot.send(()).ok();
        handle.task_handle.await??;
    }

    Ok(())
    */

    Ok(TaskHandle {
        task_handle,
        communication: Box::new(sender),
        close_shot: close_sender,
    })
}

pub async fn close_job<N>(name: N) -> Result<()>
where
    N: AsRef<str>,
{
    /*
    if let Some((_, handle)) = TASK_HANDLES.remove(name.as_ref()) {
        handle.close_shot.send(()).ok();
        handle.task_handle.await??;
    }

    Ok(())
    */
    todo!();
}

pub fn get_channel<N, T>(name: N) -> Option<UnboundedSender<T>>
where
    N: AsRef<str>,
    T: 'static,
{
    /*
    let a = TASK_HANDLES.get(name.as_ref())?;
    let chan = a.communication.downcast_ref::<UnboundedSender<T>>()?;
    Some(chan.clone())
    */
    todo!();
}
