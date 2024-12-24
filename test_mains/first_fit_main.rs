#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::{allocator, println};
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use blog_os::allocator::{ALLOCATOR, HEAP_SIZE, HEAP_START};
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // 打印初始空闲区域
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }


    println!("\nTesting memory allocation and merging...");
    
    // 分配多个内存块
    let ptr1 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    let ptr2 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    let ptr3 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    let ptr4 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    let ptr5 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    let ptr6 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(1024, 8).unwrap()) };
    
    println!("\nAfter multiple allocations:");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    // 释放不连续的内存块
    unsafe {
        ALLOCATOR.dealloc(ptr2, Layout::from_size_align(1024, 8).unwrap());
        ALLOCATOR.dealloc(ptr4, Layout::from_size_align(1024, 8).unwrap());
        ALLOCATOR.dealloc(ptr5, Layout::from_size_align(1024, 8).unwrap());
    }
    
    println!("\nAfter freeing some blocks:");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    let ptr7 = unsafe { ALLOCATOR.alloc(Layout::from_size_align(512, 8).unwrap()) };

    println!("\nAfter multiple allocations:");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    // 释放中间的内存块，触发合并
    unsafe {
        ALLOCATOR.dealloc(ptr1, Layout::from_size_align(1024, 8).unwrap());
        ALLOCATOR.dealloc(ptr3, Layout::from_size_align(1024, 8).unwrap());
        ALLOCATOR.dealloc(ptr6, Layout::from_size_align(1024, 8).unwrap());
        ALLOCATOR.dealloc(ptr7, Layout::from_size_align(512, 8).unwrap());
    }
    
    println!("\nAfter freeing middle block (should merge):");
    unsafe {
        ALLOCATOR.lock().print_free_regions();
    }

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
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
