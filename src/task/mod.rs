use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    task::{Context, Poll},
};
use executor::Executor;
use lazy_static::lazy_static;
use spin::Mutex;

// 追踪当前执行的任务
static CURRENT_TASK: AtomicUsize = AtomicUsize::new(0);
static CURRENT_PRIORITY: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// 全局执行器实例
    pub static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());
}

pub mod executor;
pub mod keyboard;
pub mod shell;
pub mod user_task;

pub const BUFFER_HEIGHT: usize = 25; // 缓冲区高度
pub const BUFFER_WIDTH: usize = 80; // 缓冲区宽度
pub const PRIORITY_NUM: usize = 3; // 优先级数量

/// 任务优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,    // 低优先级
    Normal = 1, // 普通优先级
    High = 2,   // 高优先级
}

/// 任务结构体
pub struct Task {
    id: TaskId,                                       // 任务 ID
    future: Pin<Box<dyn Future<Output = ()> + Send>>, // 任务的 Future
    priority: Priority,                               // 任务优先级
}

impl Task {
    /// 创建一个新的任务
    pub fn new(future: impl Future<Output = ()> + Send + 'static, priority: Priority) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
            priority,
        }
    }

    /// 轮询任务以推进其状态
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

/// 任务 ID 结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    /// 创建一个新的任务 ID
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
