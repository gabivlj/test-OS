use super::{Task, TaskId};

use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;

unsafe impl Send for Task {}

lazy_static! {
    static ref SPAWNED_TASKS: Arc<ArrayQueue<Task>> = Arc::new(ArrayQueue::new(500));
}

/// Proper Executor that doesn't constantly poll futures
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    spawned_tasks: Arc<ArrayQueue<Task>>,
    waker_cache: BTreeMap<TaskId, Waker>,
    // new_tasks_queue: Arc<ArrayQueue<Arc<Task>>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
            spawned_tasks: SPAWNED_TASKS.clone(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with the same ID already exists");
        }
        self.task_queue
            .push(task_id)
            .expect("queue is full. consider increasing the number of concurrent tasks");
    }

    pub fn spawn_task() -> Arc<ArrayQueue<Task>> {
        SPAWNED_TASKS.clone()
    }

    pub fn run(&mut self) -> ! {
        // use core::ops::function::Fn;
        loop {
            while let Ok(task) = self.spawned_tasks.pop() {
                self.spawn(task);
            }
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
    async fn e() -> u64 {
        3
    }

    async fn uwu() {
        use crate::println;
        println!("{}", Executor::e().await);
    }
    fn sleep_if_idle(&mut self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.task_queue.is_empty() && SPAWNED_TASKS.len() == 0 {
            enable_and_hlt();
            Executor::spawn_task().push(Task::new(Executor::uwu()));
        } else {
            interrupts::enable();
        }
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
            // new_tasks_queue,
            // external_task_queue,
            spawned_tasks,
        } = self;
        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

use alloc::task::Wake;

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
