use super::{Locked, align_up};
use crate::println;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// 首次适配分配器
pub struct FirstFitAllocator {
    head: ListNode,
}

impl FirstFitAllocator {
    /// 创建一个新的 FirstFitAllocator 实例
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// 初始化堆区域
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        let mut node = ListNode::new(heap_size);
        node.next = self.head.next.take();
        self.head.next = Some(unsafe { &mut *(heap_start as *mut ListNode) });
        unsafe {
            ptr::write(heap_start as *mut ListNode, node);
        }
    }

    /// 查找合适的空闲区域
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;

        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            }
            current = current.next.as_mut().unwrap();
        }
        None
    }

    /// 从区域中分配内存
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        Ok(alloc_start)
    }

    /// 添加一个空闲区域
    fn add_free_region(&mut self, addr: usize, size: usize) {
        let mut current = &mut self.head;
        let new_node = ListNode::new(size);
        let new_node_addr = addr;
        let new_node_end = addr + size;

        // 寻找合适的位置插入新节点
        while let Some(ref mut next) = current.next {
            if next.start_addr() == new_node_end {
                // 可以与下一个节点合并
                let mut merged = ListNode::new(size + next.size);
                merged.next = next.next.take();
                unsafe {
                    (addr as *mut ListNode).write(merged);
                }
                current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
                return;
            } else if next.end_addr() == new_node_addr {
                // 可以与当前节点合并
                next.size += size;
                // 检查是否可以与下一个节点合并
                let next_end_addr = next.end_addr();
                if let Some(ref mut next_next) = next.next {
                    if next_end_addr == next_next.start_addr() {
                        next.size += next_next.size;
                        next.next = next_next.next.take();
                    }
                }
                return;
            } else if next.start_addr() > new_node_addr {
                // 在这里插入新节点
                let mut node = ListNode::new(size);
                node.next = current.next.take();
                unsafe {
                    (addr as *mut ListNode).write(node);
                }
                current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
                return;
            }
            current = current.next.as_mut().unwrap();
        }

        // 如果没有找到合适的位置，添加到链表末尾
        let mut node = ListNode::new(size);
        node.next = None;
        unsafe {
            (addr as *mut ListNode).write(node);
        }
        current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    }

    /// 打印当前的空闲区域
    pub fn print_free_regions(&self) {
        let mut current = &self.head;
        println!("Free regions:");
        while let Some(region) = current.next.as_ref() {
            println!(
                "  Address: {:#x}, Size: {}",
                region.start_addr(),
                region.size
            );
            current = region;
        }
    }

    /// 计算大小和对齐
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<FirstFitAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = FirstFitAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start + size;
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = FirstFitAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
