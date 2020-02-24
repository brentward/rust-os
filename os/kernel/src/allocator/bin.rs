use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    current: usize,
    end: usize,
    block_sizes: [usize; 10],
    sized_lists: [LinkedList; 10],
    // FIXME: Add the necessary fields.
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            current: start,
            end,
            block_sizes: [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096],
            sized_lists: [LinkedList::new(); 10],
        }
    }

    fn get_size_index(&self, layout: &Layout) -> Option<usize> {
        let required_size = layout.size().max(layout.align());
        self.block_sizes.iter().position(| &size | size >= required_size)
    }

    fn pop_from_above(&mut self, index: usize) -> Option<*mut usize> {
        if (index + 2) <= self.block_sizes.len() {
            if !self.sized_lists[index + 1].is_empty() {
                return self.sized_lists[index + 1].pop()
            }
        }
        None
    }

    fn split_addr(addr: *mut usize, size: usize) -> (*mut usize, *mut usize) {
        let new_addr = unsafe { *addr as usize  + size };
        let new_ptr = new_addr as *mut usize;
        (addr, new_ptr)
    }

    fn push_from_above(&mut self, index: usize, low_item: *mut usize, high_item: *mut usize)  {
        unsafe {
            self.sized_lists[index].push(low_item);
            self.sized_lists[index].push(high_item);
        }
    }

    fn populate_from_above(&mut self, index: usize) -> Option<()> {
        if self.block_sizes.len() <= index {
            return None
        }
        while self.sized_lists[index].is_empty() {
            match self.populate_from_above(index + 1) {
                Some(_) => (),
                None => return None,
            }
        }
        match self.pop_from_above(index) {
            Some(addr) => {
                let (low_addr, high_addr) = Allocator::split_addr(
                    addr,
                    self.block_sizes[index]
                );
                self.push_from_above(index, low_addr, high_addr);
                Some(())
            }
            None => None,
        }
    }


    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        match self.get_size_index(&layout) {
            Some(index) => {
                match self.sized_lists[index].pop() {
                    Some(addr) => Ok(addr as *mut u8),
                    None => {
                        match self.populate_from_above(index) {
                            Some(_) => {
                                match self.sized_lists[index].pop() {
                                    Some(addr) => Ok(addr as *mut u8),
                                    None => panic!("item in list should be guarenteed")
                                }
                            },
                            None => {
                                let aligned_addr = align_up(self.current, self.block_sizes[index]);
                                if aligned_addr + self.block_sizes[index] > self.end {
                                    Err(AllocErr::Exhausted { request: layout })
                                } else {
                                    self.current = aligned_addr + self.block_sizes[index];
                                    Ok(aligned_addr as *mut u8)
                                }
                            }
                        }
                    }
                }
            }
            None => {
                let aligned_addr = align_up(self.current, layout.align());
                if aligned_addr + layout.size() > self.end {
                    Err(AllocErr::Exhausted { request: layout })
                } else {
                    self.current = aligned_addr + layout.size();
                    Ok(aligned_addr as *mut u8)
                }
            }
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        match self.get_size_index(&layout) {
            Some(index) => unsafe {
                self.sized_lists[index].push(ptr as *mut usize)
            }
            None => (),
        }
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
