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

    println!("\nTesting worst-fit algorithm:");
    println!("Step 1: Allocate several blocks with different sizes");

    let layouts = [
        Layout::from_size_align(1024, 8).unwrap(),
        Layout::from_size_align(4096, 8).unwrap(),
        Layout::from_size_align(2048, 8).unwrap(),
    ];
    let mut blocks = Vec::new();

    for layout in layouts.iter() {
        let ptr = unsafe { ALLOCATOR.alloc(*layout) };
        println!("Allocated size {}", layout.size());
        unsafe { ALLOCATOR.lock().print_free_regions(); }
        blocks.push((ptr, layout));
    }

    println!("\nStep 2: Free the middle block to create a large free area");
    unsafe {
        ALLOCATOR.dealloc(blocks[1].0, *blocks[1].1);
    }
    unsafe { ALLOCATOR.lock().print_free_regions(); }

    println!("\nStep 3: Allocate a new block which should pick the largest free space");
    let test_layout = Layout::from_size_align(3000, 8).unwrap();
    let ptr = unsafe { ALLOCATOR.alloc(test_layout) };
    println!("Allocated size {}", test_layout.size());
    unsafe { ALLOCATOR.lock().print_free_regions(); }

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
