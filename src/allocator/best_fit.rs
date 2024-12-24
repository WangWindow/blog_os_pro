use super::{align_up, Locked};
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

pub struct BestFitAllocator {
    head: ListNode,
}

impl BestFitAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        let mut node = ListNode::new(heap_size);
        node.next = self.head.next.take();
        self.head.next = Some(unsafe { &mut *(heap_start as *mut ListNode) });
        unsafe {
            ptr::write(heap_start as *mut ListNode, node);
        }
    }

    // fn find_best_fit(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
    //     let mut current = &mut self.head;
    //     let mut best_region: Option<(&mut ListNode, &'static mut ListNode, usize)> = None;
    //     let mut best_size = usize::MAX;

    //     // 第一遍：找到最合适的块
    //     while let Some(ref mut region) = current.next {
    //         if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
    //             let excess_size = region.size - (alloc_start - region.start_addr()) - size;
    //             if excess_size < best_size {
    //                 best_size = excess_size;
    //                 best_region = Some((current, unsafe { &mut *(region as *mut ListNode) }, alloc_start));
    //             }
    //         }
    //         current = current.next.as_mut().unwrap();
    //     }

    //     // 如果找到了合适的块，进行分配
    //     if let Some((prev, region, alloc_start)) = best_region {
    //         let next = region.next.take();
    //         let ret = Some((prev.next.take().unwrap(), alloc_start));
    //         prev.next = next;
    //         return ret;
    //     }
    //     None
    // }
    
    // fn find_best_fit(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
    //     let mut current = &mut self.head;
    //     let mut best_region: Option<(*mut ListNode, usize)> = None;
    //     let mut best_size = usize::MAX;

    //     // 遍历所有空闲区域，找到最佳适应的区域
    //     while let Some(ref mut region) = current.next {
    //         if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
    //             let excess_size = region.size - (alloc_start - region.start_addr()) - size;
    //             if excess_size < best_size {
    //                 best_size = excess_size;
    //                 best_region = Some((current as *mut ListNode, alloc_start));
    //             }
    //         }
    //         // 安全地移动到下一个节点
    //         current = current.next.as_mut().unwrap();
    //     }

    //     // 如果找到了合适的块，进行分配
    //     if let Some((prev_ptr, alloc_start)) = best_region {
    //         unsafe {
    //             // 将原始指针转换回可变引用
    //             let prev = &mut *prev_ptr;

    //             // 拿到要分配的区域
    //             let mut region = prev.next.take().unwrap();

    //             // 记录下一个区域
    //             let next = region.next.take();

    //             // 更新前一个节点的 next 指针
    //             prev.next = next;

    //             // 返回分配地址
    //             Some((region as *mut ListNode, alloc_start))
    //         }
    //     } else {
    //         None
    //     }
    // }
    fn find_best_fit(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        let mut best_region: Option<(*mut ListNode, usize)> = None;
        let mut best_size = usize::MAX;
    
        // 遍历所有空闲区域，找到最佳适应的区域
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                let excess_size = region.size - (alloc_start - region.start_addr()) - size;
                if excess_size < best_size {
                    best_size = excess_size;
                    best_region = Some((current as *mut ListNode, alloc_start));
                }
            }
            // 安全地移动到下一个节点
            current = current.next.as_mut().unwrap();
        }
    
        // 如果找到了最佳区域，进行分配
        if let Some((prev, alloc_start)) = best_region {
            unsafe {
                let prev_mut = &mut *prev;
                let region = prev_mut.next.take().unwrap();
                let next = region.next.take();
                prev_mut.next = next;
                Some((region, alloc_start))
            }
        } else {
            None
        }
    }
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        Ok(alloc_start)
    }

    // fn add_free_region(&mut self, addr: usize, size: usize) {
    //     let mut current = &mut self.head;
    //     let new_node_addr = addr;
    //     let new_node_end = addr + size;

    //     // 合并相邻的空闲块
    //     while let Some(ref mut next) = current.next {
    //         if next.start_addr() == new_node_end {
    //             // 与后一个块合并
    //             let mut merged = ListNode::new(size + next.size);
    //             merged.next = next.next.take();
    //             unsafe {
    //                 (addr as *mut ListNode).write(merged);
    //             }
    //             current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    //             return;
    //         } else if next.end_addr() == new_node_addr {
    //             // 与前一个块合并
    //             next.size += size;
    //             // 检查是否可以与下一个块合并
    //             if let Some(ref mut next_next) = next.next {
    //                 if next.end_addr() == next_next.start_addr() {
    //                     next.size += next_next.size;
    //                     next.next = next_next.next.take();
    //                 }
    //             }
    //             return;
    //         } else if next.start_addr() > new_node_addr {
    //             // 在正确的位置插入新节点
    //             let mut node = ListNode::new(size);
    //             node.next = current.next.take();
    //             unsafe {
    //                 (addr as *mut ListNode).write(node);
    //             }
    //             current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    //             return;
    //         }
    //         current = current.next.as_mut().unwrap();
    //     }

    //     // 添加到链表末尾
    //     let mut node = ListNode::new(size);
    //     node.next = None;
    //     unsafe {
    //         (addr as *mut ListNode).write(node);
    //     }
    //     current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    // }
    // fn add_free_region(&mut self, addr: usize, size: usize) {
    //     let mut current = &mut self.head;
    //     let new_node_addr = addr;
    //     let new_node_end = addr + size;

    //     // 合并相邻的空闲块
    //     while let Some(ref mut next) = current.next {
    //         if next.start_addr() == new_node_end {
    //             // 与后一个块合并
    //             let mut merged = ListNode::new(size + next.size);
    //             merged.next = next.next.take();
    //             unsafe {
    //                 (addr as *mut ListNode).write(merged);
    //             }
    //             current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    //             return;
    //         } else if next.end_addr() == new_node_addr {
    //             // 与前一个块合并
    //             next.size += size;
    //             // 检查是否可以与下一个块合并
    //             if let Some(ref mut next_next) = next.next {
    //                 if next.end_addr() == next_next.start_addr() {
    //                     next.size += next_next.size;
    //                     next.next = next_next.next.take();
    //                 }
    //             }
    //             return;
    //         } else if next.start_addr() > new_node_addr {
    //             // 在正确的位置插入新节点
    //             let mut node = ListNode::new(size);
    //             node.next = current.next.take();
    //             unsafe {
    //                 (addr as *mut ListNode).write(node);
    //             }
    //             current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    //             return;
    //         }
    //         current = current.next.as_mut().unwrap();
    //     }

    //     // 添加到链表末尾
    //     let mut node = ListNode::new(size);
    //     node.next = None;
    //     unsafe {
    //         (addr as *mut ListNode).write(node);
    //     }
    //     current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    // }
    fn add_free_region(&mut self, addr: usize, size: usize) {
        // 与现有的 FirstFitAllocator 合并逻辑相同
        let mut current = &mut self.head;
        let new_node = ListNode::new(size);
        let new_node_addr = addr;
        let new_node_end = addr + size;

        while let Some(ref mut next) = current.next {
            if next.start_addr() == new_node_end {
                // 前向合并
                let mut merged = ListNode::new(size + next.size);
                merged.next = next.next.take();
                unsafe {
                    (addr as *mut ListNode).write(merged);
                }
                current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
                return;
            } else if next.end_addr() == new_node_addr {
                // 后向合并
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
                // 插入新节点
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

        // 添加到末尾
        let mut node = ListNode::new(size);
        node.next = None;
        unsafe {
            (addr as *mut ListNode).write(node);
        }
        current.next = Some(unsafe { &mut *(addr as *mut ListNode) });
    }


    pub fn print_free_regions(&self) {
        let mut current = &self.head;
        println!("Free regions (Best Fit):");
        let mut total_free = 0;
        let mut count = 0;
        while let Some(region) = current.next.as_ref() {
            println!(
                "  Region {}: Address: {:#x}, Size: {} bytes",
                count + 1,
                region.start_addr(),
                region.size
            );
            total_free += region.size;
            count += 1;
            current = region;
        }
        println!("  Total: {} regions, {} bytes free", count, total_free);
    }
}

unsafe impl GlobalAlloc for Locked<BestFitAllocator> {
    // unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    //     let (size, align) = BestFitAllocator::size_align(layout);
    //     let mut allocator = self.lock();

    //     if let Some((region, alloc_start)) = allocator.find_best_fit(size, align) {
    //         let alloc_end = alloc_start + size;
    //         let excess_size = region.end_addr() - alloc_end;
    //         if excess_size > 0 {
    //             allocator.add_free_region(alloc_end, excess_size);
    //         }
    //         alloc_start as *mut u8
    //     } else {
    //         ptr::null_mut()
    //     }
    // }
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = BestFitAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_best_fit(size, align) {
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
        let (size, _) = BestFitAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}

impl BestFitAllocator {
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}
