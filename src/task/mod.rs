use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

/// 任务优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,    // 低优先级
    Normal = 1, // 普通优先级
    High = 2,   // 高优先级
}

/// 任务结构体
pub struct Task {
    id: TaskId,                                // 任务 ID
    future: Pin<Box<dyn Future<Output = ()>>>, // 任务的 Future
    priority: Priority,                        // 任务优先级
}

impl Task {
    /// 创建一个新的任务
    ///
    /// # 参数
    ///
    /// - `future`: 任务的 Future
    /// - `priority`: 任务优先级
    ///
    /// # 返回值
    ///
    /// 返回一个新的 Task 实例
    pub fn new(future: impl Future<Output = ()> + 'static, priority: Priority) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
            priority,
        }
    }

    /// 轮询任务以推进其状态
    ///
    /// # 参数
    ///
    /// - `context`: 任务的上下文
    ///
    /// # 返回值
    ///
    /// 返回任务的轮询状态
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

/// 任务 ID 结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    /// 创建一个新的任务 ID
    ///
    /// # 返回值
    ///
    /// 返回一个新的 TaskId 实例
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
