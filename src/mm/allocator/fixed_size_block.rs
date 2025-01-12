use super::Locked;
use crate::println;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};

/// 用于分配的块大小
///
/// 这些大小必须是2的幂，因为它们也用作块对齐（对齐必须始终是2的幂）
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// 为给定的布局选择一个合适的块大小
///
/// 返回一个`BLOCK_SIZES`数组的索引
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

/// 单链表中的节点
struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// 固定大小的块分配器，可以容纳不同大小的固定数量的块
pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    /// 创建一个空的FixedSizeBlockAllocator
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// 使用给定的堆边界初始化分配器
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// 使用回退分配器分配
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    /// 打印所有空闲区域的信息
    pub unsafe fn print_free_regions(&mut self) {
        println!("Fixed-Size Block Allocator Status:");

        let mut total_blocks = 0;
        let mut total_memory = 0;

        // 遍历所有块大小
        for (i, &size) in BLOCK_SIZES.iter().enumerate() {
            let mut count = 0;
            let mut current = self.list_heads[i].as_deref_mut();

            // 计算每个大小的空闲块数量
            while let Some(node) = current {
                count += 1;
                current = node.next.as_deref_mut();
            }

            // 打印每个大小的统计信息
            println!("  Size {} bytes: {} free blocks", size, count);
            total_blocks += count;
            total_memory += count * size;
        }

        // 打印总体统计信息
        println!(
            "  Total: {} free blocks, {} bytes free",
            total_blocks, total_memory
        );
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /// 分配使用块列表
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are a power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout),
        }
    }

    /// 使用块列表释放
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and alignment required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
