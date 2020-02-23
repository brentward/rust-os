use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

// const BLOCK_SIZES: &[usize; 10] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
//
// fn get_size_index(layout: &Layout) -> Option<usize> {
//     let required_size = layout.size().max(layout.align());
//     BLOCK_SIZES.iter().contains(|&size| required_size >= size)
// }

/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    current: usize,
    end: usize,
    block_sizes: [usize; 10],
    sized_lists: [LinkedList; 10],
    // default_allocator: LinkedList,
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
            // default_allocator: LinkedList::new(),
        }
    }

    fn get_size_index(&self, layout: &Layout) -> Option<usize> {
        let required_size = layout.size().max(layout.align());
        self.block_sizes.iter().position(| &size | size >= required_size)
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
                    Some(addr) => return Ok(addr as *mut u8),
                    None => {
                        // let size = self.block_sizes[index];
                        // let align = size;
                        // let layout = Layout::from_size_align(size, align)
                        //     .expect("New layout from size failed");
                        let aligned_addr = align_up(self.current, layout.align());
                        if aligned_addr + self.block_sizes[index] > self.end {
                            Err(AllocErr::Exhausted { request: layout })
                        } else {
                            self.current = aligned_addr + self.block_sizes[index];
                            Ok(aligned_addr as *mut u8)
                        }
                    },
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

                // match self.default_allocator.pop() {
                //     Some(addr) => {
                //
                //     },
                //     None => {
                //         // let size = self.block_sizes[index];
                //         // let align = size;
                //         // let layout = Layout::from_size_align(size, align)
                //         //     .expect("New layout from size failed");
                //         let aligned_addr = align_up(self.current, layout.align());
                //         if aligned_addr + layout.size() > self.end {
                //             Err(AllocErr::Exhausted { request: layout })
                //         } else {
                //             self.current = aligned_addr + layout.size();
                //             Ok(aligned_addr as *mut u8)
                //         }
                //     },
                // }
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
            },
            None => (),
        }

        // unimplemented!("bin deallocation")
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
