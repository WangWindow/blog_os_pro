#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use alloc::vec::Vec;
use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::allocator::ALLOCATOR;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    println!("\n测试最佳适应算法:");
    println!("步骤1: 分配多个不同大小的块以创建碎片");

    // 创建多个不同大小的分配请求
    let layouts = [
        Layout::from_size_align(1024, 8).unwrap(), // 小块
        // Layout::from_size_align(5000, 8).unwrap(),  // 大块
        Layout::from_size_align(2048, 8).unwrap(), // 中等块
        // Layout::from_size_align(3000, 8).unwrap(),  // 中大块
        Layout::from_size_align(512, 8).unwrap(), // 很小的块
    ];

    let mut ptrs = Vec::new();
    for (i, layout) in layouts.iter().enumerate() {
        let ptr = unsafe { ALLOCATOR.alloc(layout.clone()) };
        println!("\n分配第{}个块 (大小: {} bytes):", i + 1, layout.size());
        unsafe {
            ALLOCATOR.lock().print_free_regions();
        }
        ptrs.push((ptr, layout));
    }
    println!("okokokok");
    println!("\n步骤2: 释放一些块来创建不同大小的空闲区");
    // 释放一些块以创建空闲空间
    unsafe {
        ALLOCATOR.dealloc(ptrs[1].0, *ptrs[1].1); // 释放5000字节
        ALLOCATOR.dealloc(ptrs[3].0, *ptrs[3].1); // 释放3000字节
    }
    println!("\n释放后的空闲区域:");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    println!("\n步骤3: 测试最佳适应 - 分配一些特定大小的块");
    // 尝试分配特定大小的块以测试最佳适应
    let test_layouts = [
        Layout::from_size_align(2800, 8).unwrap(), // 应该使用3000的空闲块
        Layout::from_size_align(1500, 8).unwrap(), // 应该使用剩余较小的块
        Layout::from_size_align(4000, 8).unwrap(), // 应该使用5000的空闲块
    ];

    let mut test_ptrs = Vec::new();
    for (i, layout) in test_layouts.iter().enumerate() {
        let ptr = unsafe { ALLOCATOR.alloc(layout.clone()) };
        println!("\n分配测试块{} (大小: {} bytes):", i + 1, layout.size());
        unsafe {
            ALLOCATOR.lock().print_free_regions();
        }
        test_ptrs.push((ptr, layout));
    }

    println!("\n步骤4: 清理所有分配");
    // 释放所有剩余的块
    for (ptr, layout) in ptrs.iter().chain(test_ptrs.iter()) {
        if !(*ptr).is_null() {
            unsafe {
                ALLOCATOR.dealloc(*ptr, **layout);
            }
        }
    }
    println!("\n清理后的状态:");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    #[cfg(test)]
    test_main();

    loop{

    }
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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
