use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{
    sync::atomic::{AtomicU8, Ordering},
    task::{Context, Poll, Waker},
};

use crossbeam_queue::ArrayQueue;
use futures_util::{future::BoxFuture, task, task::ArcWake};
use spin::{Mutex, Once};

pub static ASYNC_EXECUTOR: Once<Mutex<TaskExecutor>> = Once::new();

const MAX_TASKS: usize = 128;

pub fn init() {
    ASYNC_EXECUTOR.call_once(|| Mutex::new(TaskExecutor::new()));
}

pub struct Task {
    future: BoxFuture<'static, ()>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }
}

struct TaskWaker {
    id: u8,
    task_queue: Arc<ArrayQueue<u8>>,
}

impl TaskWaker {
    fn new_waker(id: u8, task_queue: Arc<ArrayQueue<u8>>) -> Waker {
        task::waker(Arc::new(Self { id, task_queue }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.id).expect("task queue full")
    }
}

impl ArcWake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }

    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.wake_task()
    }
}

pub struct TaskExecutor {
    tasks: BTreeMap<u8, Task>,
    task_queue: Arc<ArrayQueue<u8>>,
    waker_cache: BTreeMap<u8, Waker>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(MAX_TASKS)),
            waker_cache: BTreeMap::new(),
        }
    }

    // TODO: Make spawning not require mut self
    pub fn spawn(&mut self, task: Task) {
        static NEXT_TASK_ID: AtomicU8 = AtomicU8::new(0);

        let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);

        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same id is already present")
        }

        self.task_queue.push(task_id).expect("task queue is full")
    }

    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(t) => t,
                None => continue, // task no longer exists
            };

            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new_waker(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);

            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(_) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();

        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt()
        } else {
            interrupts::enable()
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
}
