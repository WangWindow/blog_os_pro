#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Priority, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    // 创建执行器
    let mut executor = Executor::new();

    // 创建不同优先级的任务
    executor.spawn(Task::new(
        example_task("Task1: Highh Priority"),
        Priority::High,
    ));
    executor.spawn(Task::new(
        example_task("Task2: Normal Priority"),
        Priority::Normal,
    ));
    executor.spawn(Task::new(
        example_task("Task3: Low Priority "),
        Priority::Low,
    ));

    executor.spawn(Task::new(
        example_task("Task4: Normal Priority "),
        Priority::Normal,
    ));

    executor.spawn(Task::new(
        example_task("Task5: High Priority "),
        Priority::High,
    ));

    // 启动键盘中断处理任务
    // executor.spawn(Task::new(keyboard::print_keypresses(), Priority::Low));

    // 运行执行器
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

// 一个简单的异步任务
async fn example_task(id: &str) {
    println!("Task {} start", id);
    for i in 0..3 {
        println!("Task {} step {}", id, i);
    }
    println!("Task {} down", id);
}
