#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

pub const BUFFER_HEIGHT: usize = 25; // 缓冲区高度

lazy_static! {
    /// Global file system
    ///
    /// 全局文件系统
    pub static ref FILESYSTEM: Mutex<io::fs::FileSystem> = Mutex::new(io::fs::FileSystem::new());
}

pub mod boot;
pub mod int;
pub mod io;
pub mod mm;
pub mod task;

/// Initialize the kernel
///
/// 初始化内核
pub fn init() {
    boot::gdt::init();
    int::init_idt();
    unsafe { int::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// Entry point for `cargo test`
///
/// `cargo test` 的入口点
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Test panic handler
///
/// 测试 panic 处理函数
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Enum representing QEMU exit codes
///
/// 表示 QEMU 退出码的枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exit QEMU with the given exit code
///
/// 使用给定的退出码退出 QEMU
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Loop that halts the CPU
///
/// 使 CPU 进入循环休眠状态
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
///
/// `cargo xtest` 的入口点
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
