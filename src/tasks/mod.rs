pub mod deletion;
pub mod identification;

use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use tokio::task::{self, JoinHandle};

pub use self::{deletion::deletion, identification::identification};

#[derive(Clone)]
pub struct TaskStatus {
    active: Arc<AtomicBool>,
}

impl TaskStatus {
    fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn stop(&self) {
        self.active.store(false, Ordering::Release);
    }
}

pin_project_lite::pin_project! {
    pub struct Task {
        status: TaskStatus,
        #[pin]
        handle: JoinHandle<()>,
    }
}

impl Task {
    pub fn spawn<T, F>(task: T) -> Self
    where
        T: FnOnce(TaskStatus) -> F + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let status = TaskStatus::new();

        Self {
            status: status.clone(),
            handle: task::spawn(task(status)),
        }
    }
}

impl Future for Task {
    type Output = <JoinHandle<()> as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.status.stop();

        let this = self.project();

        this.handle.poll(cx)
    }
}
