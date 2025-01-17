use crate::{println, time};

/// 一个简单的打印任务
pub async fn print_task() {
    let mut i = 0;
    loop {
        println!("print_task: {}", i);
        i += 1;
        time::sleep(1);
    }
}

/// 一个定时打印任务
pub async fn timed_print_task(time: u64) {
    let mut i = 0;
    loop {
        println!("timed_print_task: {}", i);
        i += 1;
        time::sleep(time);
    }
}

/// 一个有限时间的任务
pub async fn limited_time_task(time: u64) {
    let mut i = 0;
    while i < time {
        println!("limited_time_task: {}", i);
        i += 1;
        time::sleep(1);
    }
}
