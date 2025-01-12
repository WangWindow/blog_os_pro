use super::{CURRENT_PRIORITY, CURRENT_TASK, PRIORITY_NUM, Priority, Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::{
    sync::atomic::Ordering,
    task::{Context, Poll, Waker},
};
use crossbeam_queue::{ArrayQueue, SegQueue};

/// Executor 负责管理和调度任务
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>, // 存储任务 ID 与对应任务的映射
    task_queues: [Arc<ArrayQueue<TaskId>>; PRIORITY_NUM], // 不同优先级的任务队列，使用数组存储
    waker_cache: BTreeMap<TaskId, Waker>, // 缓存每个任务的 Waker，避免重复创建
    new_tasks: Arc<SegQueue<Task>>, // 新任务队列，用于从外部添加新任务
}

impl Executor {
    /// 创建一个新的 Executor 实例，并初始化任务队列和 Waker 缓存
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queues: [
                Arc::new(ArrayQueue::new(100)), // 低优先级队列
                Arc::new(ArrayQueue::new(100)), // 普通优先级队列
                Arc::new(ArrayQueue::new(100)), // 高优先级队列
            ],
            waker_cache: BTreeMap::new(),
            new_tasks: Arc::new(SegQueue::new()),
        }
    }

    /// 创建一个 Spawner 实例，用于从外部添加新任务
    pub fn spawner(&self) -> Spawner {
        Spawner {
            new_tasks: self.new_tasks.clone(),
        }
    }

    /// 启动一个新任务，将其插入到对应优先级的队列中
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        let priority = task.priority as usize;
        // 将任务插入到 tasks 映射中，如果任务 ID 已存在则触发恐慌
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        // 将任务 ID 推入对应优先级的任务队列中
        self.task_queues[priority]
            .push(task_id)
            .expect("queue full");
    }

    /// 持续运行准备好的任务，并在空闲时休眠
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks(); // 执行所有准备好的任务
            self.sleep_if_idle(); // 如果没有任务，休眠以节省资源
        }
    }

    /// 如果没有准备好的任务，则将 CPU 置于低功耗状态
    pub fn sleep_if_idle(&self) {
        use x86_64::instructions::hlt;
        hlt(); // 使 CPU 进入暂停状态，直到下一个中断
    }

    /// 运行所有准备好的任务，通过轮询它们并管理其状态
    fn run_ready_tasks(&mut self) {
        // 按照优先级从高到低遍历任务队列
        for priority in (0..PRIORITY_NUM).rev() {
            while let Some(task_id) = self.task_queues[priority].pop() {
                // 获取对应的任务，如果任务不存在则跳过
                let task = match self.tasks.get_mut(&task_id) {
                    Some(task) => task,
                    None => continue,
                };

                // 提前获取任务的优先级和对应的队列
                let task_priority = task.priority;
                let task_queue = self.task_queues[task_priority as usize].clone();

                // 获取或创建该任务的 Waker
                let waker = self
                    .waker_cache
                    .entry(task_id)
                    .or_insert_with(|| TaskWaker::new(task_id, task_priority, task_queue));

                // 创建一个 Context 对象用于轮询任务
                let mut context = Context::from_waker(waker);
                // 轮询任务以推进其状态
                match task.poll(&mut context) {
                    Poll::Ready(()) => {
                        // 如果任务完成，移除任务和对应的 Waker
                        self.tasks.remove(&task_id);
                        self.waker_cache.remove(&task_id);
                    }
                    Poll::Pending => {
                        // 如果任务未完成，保留在队列中等待下次轮询
                    }
                }
            }
        }
    }
}

/// TaskWaker 负责通过重新将任务 ID 推入任务队列来唤醒任务
struct TaskWaker {
    task_id: TaskId,
    priority: Priority,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, priority: Priority, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            priority,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        let current_priority = CURRENT_PRIORITY.load(Ordering::Relaxed);

        // 如果当前任务优先级更高,则触发重新调度
        if self.priority as usize > current_priority {
            self.task_queue.push(self.task_id).expect("task queue full");
        }
    }
}

impl Wake for TaskWaker {
    /// 唤醒任务，将其 ID 推入任务队列中
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    /// 通过引用唤醒任务，允许多个 Waker 实例引用同一个任务
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

/// Spawner 负责从外部添加新任务
pub struct Spawner {
    new_tasks: Arc<SegQueue<Task>>,
}
impl Spawner {
    /// 添加一个新任务到新任务队列
    pub fn spawn(&self, task: Task) {
        self.new_tasks.push(task);
    }
}
