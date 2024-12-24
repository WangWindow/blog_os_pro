#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::alloc::{Layout, GlobalAlloc};
use alloc::vec::Vec;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    use blog_os::allocator::ALLOCATOR;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // println!("\nTesting Next-Fit allocation algorithm:");

    // // 分配一些内存块
    // let layout1 = Layout::from_size_align(1000, 8).unwrap();
    // let layout2 = Layout::from_size_align(2000, 8).unwrap();
    // let layout3 = Layout::from_size_align(3000, 8).unwrap();
    // println!("\nBefore any allocation:");
    // // unsafe { ALLOCATOR.lock().print_free_regions(); }
    
    // let ptr1 = unsafe { ALLOCATOR.alloc(layout1) };
    // println!("\nAfter first allocation (1000 bytes):");
    // // unsafe { ALLOCATOR.lock().print_free_regions(); }

    // let ptr2 = unsafe { ALLOCATOR.alloc(layout2) };
    // println!("\nAfter second allocation (2000 bytes):");
    // // unsafe { ALLOCATOR.lock().print_free_regions(); }

    // let ptr3 = unsafe { ALLOCATOR.alloc(layout3) };
    // println!("\nAfter third allocation (3000 bytes):");
    // // unsafe { ALLOCATOR.lock().print_free_regions(); }

    // // 释放中间的内存块
    // unsafe { 
    //     ALLOCATOR.dealloc(ptr2, layout2);
    // }
    // println!("\nAfter freeing the middle block:");
    // unsafe { ALLOCATOR.lock().print_free_regions(); }

    // // 尝试分配一个较小的块，应该从最后访问位置开始查找
    // let layout4 = Layout::from_size_align(1500, 8).unwrap();
    // let ptr4 = unsafe { ALLOCATOR.alloc(layout4) };
    // println!("\nAfter allocating 1500 bytes (should use next-fit):");
    // unsafe { ALLOCATOR.lock().print_free_regions(); }

    // // 清理所有分配
    // unsafe {
    //     ALLOCATOR.dealloc(ptr1, layout1);
    //     ALLOCATOR.dealloc(ptr3, layout3);
    //     ALLOCATOR.dealloc(ptr4, layout4);
    // }
    
    // println!("\nAfter cleanup:");
    // unsafe { ALLOCATOR.lock().print_free_regions(); }

    println!("\nTesting First-Fit with wraparound:");
    println!("Step 1: Allocate 5 blocks of different sizes sequentially");
    
    // 分配5个不同大小的块
    let layouts = [
        Layout::from_size_align(1000, 8).unwrap(),  // small
        Layout::from_size_align(4000, 8).unwrap(),  // large
        Layout::from_size_align(2000, 8).unwrap(),  // medium
        Layout::from_size_align(3000, 8).unwrap(),  // medium-large
        Layout::from_size_align(500, 8).unwrap(),   // very small
    ];

    let mut ptrs = Vec::new();
    for (i, layout) in layouts.iter().enumerate() {
        let ptr = unsafe { ALLOCATOR.alloc(layout.clone()) };
        println!("\nAllocating block {} (size: {} bytes):", i + 1, layout.size());
        unsafe { ALLOCATOR.lock().print_free_regions(); }
        ptrs.push((ptr, layout));
    }

    println!("\nStep 2: Free non-adjacent blocks to create fragmentation");
    // 释放第2和第4个块，创建不连续的空闲空间
    unsafe {
        ALLOCATOR.dealloc(ptrs[1].0, *ptrs[1].1); // 4000字节
        ALLOCATOR.dealloc(ptrs[3].0, *ptrs[3].1); // 3000字节
    }
    println!("\nFree regions after deallocation:");
    unsafe { ALLOCATOR.lock().print_free_regions(); }

    println!("\nStep 3: Test wraparound search - Allocate three blocks sequentially");
    // 尝试分配三个块，大小经过特别设计
    let test_layouts = [
        Layout::from_size_align(2500, 8).unwrap(), // 应该使用第二个空闲块(3000)
        Layout::from_size_align(3500, 8).unwrap(), // 应该使用第一个空闲块(4000)
        Layout::from_size_align(400, 8).unwrap(),  // 应该使用剩余的较小空闲块
    ];

    let mut test_ptrs = Vec::new();
    for (i, layout) in test_layouts.iter().enumerate() {
        let ptr = unsafe { ALLOCATOR.alloc(layout.clone()) };
        println!("\nAllocating test block {} (size: {} bytes):", i + 1, layout.size());
        unsafe { ALLOCATOR.lock().print_free_regions(); }
        test_ptrs.push((ptr, layout));
    }

    println!("\nStep 4: Clean up all allocations");
    // 释放所有剩余的块
    for (ptr, layout) in ptrs.iter().chain(test_ptrs.iter()) {
        if !(*ptr).is_null() {
            unsafe { ALLOCATOR.dealloc(*ptr, **layout); }
        }
    }
    println!("\nState after cleanup:");
    unsafe { ALLOCATOR.lock().print_free_regions(); }

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
