#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

// use blog_os::int::time::sleep;
use blog_os::mm::{allocator, memory, memory::BootInfoFrameAllocator};
use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(kernel_main);

/// 内核入口点
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 初始化
    blog_os::boot::welcome::show_welcome();
    println!("Hello World{}", "!");
    blog_os::init();

    // 初始化内存分配器
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 初始化堆
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    // 初始化任务
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
    // let mut count = 0;
    // println!("Count Start!");
    // loop {
    //     sleep(10);
    //     count = count + 1;
    //     println!("{}", count);
    // }
}

/// 当 panic 时调用
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
