use core::sync::atomic::{AtomicU64, Ordering};

/// 时钟计数器(毫秒)
static TIMER_COUNT: AtomicU64 = AtomicU64::new(0);

/// 更新时钟计数
pub fn tick() {
    TIMER_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// 获取系统启动后经过的毫秒数
pub fn current_time_millis() -> u64 {
    TIMER_COUNT.load(Ordering::SeqCst)
}

/// 获取格式化的运行时间字符串
pub fn uptime_str() -> alloc::string::String {
    let ms = current_time_millis();
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    alloc::format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
}
