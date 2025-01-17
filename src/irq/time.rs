use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::time::Duration;

/// 时钟计数器(毫秒)
static TIMER_COUNT: AtomicU64 = AtomicU64::new(0);

/// 是否启用定时器
static TIMER_ENABLED: AtomicBool = AtomicBool::new(false);

/// 时间单位枚举
#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

/// 定时器结构体
pub struct Timer {
    start_time: u64,
    duration: Duration,
    expired: bool,
}

impl Timer {
    /// 创建新定时器
    pub fn new(duration: Duration) -> Self {
        Self {
            start_time: current_time_millis(),
            duration,
            expired: false,
        }
    }

    /// 检查定时器是否过期
    pub fn is_expired(&mut self) -> bool {
        if self.expired {
            return true;
        }
        let now = current_time_millis();
        if now - self.start_time >= self.duration.as_millis() as u64 {
            self.expired = true;
            return true;
        }
        false
    }

    /// 重置定时器
    pub fn reset(&mut self) {
        self.start_time = current_time_millis();
        self.expired = false;
    }
}

/// 计时器结构体
pub struct Stopwatch {
    start_time: Option<u64>,
    elapsed: u64,
}

impl Stopwatch {
    /// 创建新计时器
    pub fn new() -> Self {
        Self {
            start_time: None,
            elapsed: 0,
        }
    }

    /// 开始计时
    pub fn start(&mut self) {
        self.start_time = Some(current_time_millis());
    }

    /// 停止计时
    pub fn stop(&mut self) {
        if let Some(start) = self.start_time {
            self.elapsed += current_time_millis() - start;
            self.start_time = None;
        }
    }

    /// 重置计时器
    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = 0;
    }

    /// 获取经过时间
    pub fn elapsed(&self) -> u64 {
        self.elapsed
            + self
                .start_time
                .map_or(0, |start| current_time_millis() - start)
    }
}

/// 时钟计数加一
pub fn tick() {
    TIMER_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// 获取当前时间(毫秒)
pub fn current_time_millis() -> u64 {
    TIMER_COUNT.load(Ordering::Relaxed)
}

/// 获取指定单位的当前时间
pub fn current_time(unit: TimeUnit) -> u64 {
    let millis = current_time_millis();
    match unit {
        TimeUnit::Milliseconds => millis,
        TimeUnit::Seconds => millis / 1000,
        TimeUnit::Minutes => millis / (1000 * 60),
        TimeUnit::Hours => millis / (1000 * 60 * 60),
    }
}

/// 格式化时间为字符串
pub fn format_time(millis: u64) -> String {
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours % 24,
        minutes % 60,
        seconds % 60,
        millis % 1000
    )
}

/// 休眠指定毫秒数
pub fn sleep(ms: u64) {
    let end = current_time_millis() + ms;
    while current_time_millis() < end {
        core::hint::spin_loop();
    }
}

/// 启用定时器
pub fn enable_timer() {
    TIMER_ENABLED.store(true, Ordering::SeqCst);
}

/// 禁用定时器
pub fn disable_timer() {
    TIMER_ENABLED.store(false, Ordering::SeqCst);
}

/// 检查定时器是否启用
pub fn is_timer_enabled() -> bool {
    TIMER_ENABLED.load(Ordering::SeqCst)
}

// use core::sync::atomic::{AtomicU64, Ordering};

// /// 时钟计数器(毫秒)
// static TIMER_COUNT: AtomicU64 = AtomicU64::new(0);

// /// 更新时钟计数
// pub fn tick() {
//     TIMER_COUNT.fetch_add(1, Ordering::SeqCst);
// }

// /// 获取系统启动后经过的毫秒数
// pub fn current_time_millis() -> u64 {
//     TIMER_COUNT.load(Ordering::SeqCst)
// }

// /// 获取格式化的运行时间字符串
// pub fn uptime_str() -> alloc::string::String {
//     let ms = current_time_millis();
//     let seconds = ms / 1000;
//     let minutes = seconds / 60;
//     let hours = minutes / 60;

//     alloc::format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
// }

// /// 阻塞指定的时间( time 个时钟周期)
// pub fn sleep(time: u64) {
//     let current_time = current_time_millis();

//     while current_time_millis() - current_time < time {}
// }
