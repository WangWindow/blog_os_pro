use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A simple allocator that allocates memory linearly and ignores deallocation.
///
/// This allocator is useful for setting up a more complex allocator.
/// It is not intended to be used for longer periods of time, because it
/// just leaks memory.
///
/// Bump 分配器是一种简单的分配器，它会线性分配内存并忽略释放。
/// 这种分配器对于设置更复杂的分配器非常有用。
/// 它不适合长时间使用，因为它会泄漏内存。
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    ///
    /// 创建一个新的空 bump 分配器。
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    ///
    /// 使用给定的堆边界初始化 bump 分配器。
    ///
    /// 这个方法是不安全的，因为调用者必须确保给定的内存范围未使用。
    /// 而且，这个方法只能调用一次。
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start.saturating_add(heap_size);
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocates memory as described by the given layout.
    /// Returns a pointer to the start of the allocated block of memory.
    ///
    /// 根据给定的布局分配内存。
    /// 返回指向分配的内存块的起始位置的指针。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Deallocates the memory referenced by the given pointer.
    /// The pointer must be a pointer that was returned by a previous call to `alloc`.
    ///
    /// 释放给定指针引用的内存。
    /// 指针必须是之前调用 `alloc` 返回的指针。
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
