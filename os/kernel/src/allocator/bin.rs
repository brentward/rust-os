use std::{fmt, mem, usize};
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

const BLOCK_SIZE_COUNT: usize = mem::size_of::<usize>() * 8 - 3;

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    total_mem: usize,
    max_block_size: usize,
    sized_block_count: usize,
    current: usize,
    end: usize,
    sized_blocks: [LinkedList; BLOCK_SIZE_COUNT],
}
impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let total_mem = end - start;
        let max_block_size = 1 << (mem::size_of::<usize>() * 8 - total_mem.leading_zeros() as usize - 1);
        let sized_block_count = (max_block_size as u64).trailing_zeros() as usize - 2;
        // let starting_address = align_up(start, max_power_of_two);
        // let (max_block_size, sized_block_count, starting_address) = if end - starting_address >= max_power_of_two {
        //     (max_power_of_two, sized_block_count, starting_address)
        // } else {
        //     let sized_block_count = sized_block_count - 1;
        //     let max_block_size = Allocator::size_of_block(sized_block_count - 1);
        //     let starting_address = align_up(start, max_block_size);
        //     (max_block_size, sized_block_count, starting_address)
        // };
        let mut sized_blocks = [LinkedList::new(); BLOCK_SIZE_COUNT];
        // let current = starting_address + max_block_size;
        // unsafe { sized_blocks[sized_block_count - 1].push(starting_address as *mut usize) };

        Allocator {
            total_mem,
            max_block_size,
            sized_block_count,
            current: start,
            end,
            sized_blocks,
        }
        // match allocator.populate_from_above(0) {
        //     Some(_) => allocator,
        //     None => panic!("Nothing was returned from populate from above"),
        // }
    }

    fn size_of_block(index: usize) -> usize {
        2usize.pow(index as u32 + 3)
    }

    fn size_list_index_for_layout(&self, layout: &Layout) -> usize {
        let required_size = layout.size().max(layout.align());
        for index in 0..self.sized_blocks.len() {
            if Allocator::size_of_block(index) >= required_size{
                return index
            }
        }
        panic!("layout will cause memory address overflow");
    }

    /// Traverses up the size_blocks array from the `index` location and checks
    /// recursively for unallocated  blocks. If it finds any blocks that are larger
    /// than the requested size it pop them from the list, split them into the next
    /// lowest block size and them push them into the corresponding size_block list
    /// down tohe list at the `index` location.
    ///
    /// The effect of this that all lists between `index` and the next highest list
    /// with that is not empty will get 1 item and the list at `index` will get two.
    fn populate_from_above(&mut self, index: usize, layout: &Layout) -> Option<*mut usize> {
        if index < self.sized_block_count {
            if self.sized_blocks[index + 1].is_empty() {
                match self.populate_from_above(index + 1, layout) {
                    Some(addr) => {
                        match self.sized_blocks[index + 1].pop() {
                            Some(addr) => {
                                let (low, high) = unsafe {
                                    split_addr(addr, Allocator::size_of_block(index))
                                };
                                let low_align = align_up(low as usize, layout.align());
                                let high_align = align_up(high as usize, layout.align());
                                let closest = if low_align - low as usize <= high_align - high as usize {
                                    unsafe {
                                        self.sized_blocks[index].push(high);
                                        low
                                    }
                                } else {
                                    unsafe {
                                        self.sized_blocks[index].push(low);
                                        high
                                    }
                                };
                                return Some(closest);
                            }
                            None => { return None; }
                        }
                    }
                    None => { return None; }
                }
            }
            None
        } else {
            None
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
        let index =  self.size_list_index_for_layout(&layout);
        for node in self.sized_blocks[index].iter_mut() {
            if has_alignment(node.value() as usize, layout.align()) {
                return Ok(node.pop() as *mut u8)
            }
        }
        let aligned_addr = align_up(self.current, Allocator::size_of_block(index));
        match self.populate_from_above(index, &layout) {
            Some(addr) => {
                if has_alignment(addr as usize, layout.align()) {
                    Ok(addr as *mut u8)
                } else {
                    Err(AllocErr::Exhausted { request: layout })
                }
                // match self.sized_blocks[index].pop() {
                //     Some(addr) => Ok(addr as *mut u8),
                //     None => panic!("populate from above is not working correctly"),
                // }
            },
            None => {
                if aligned_addr + Allocator::size_of_block(index) > self.end {
                    Err(AllocErr::Exhausted { request: layout })
                } else {
                    self.current = aligned_addr + Allocator::size_of_block(index);
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
        let index = self.size_list_index_for_layout(&layout);
        unsafe {
            self.sized_blocks[index].push(ptr as *mut usize)
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Allocator {{")?;
        writeln!(f, "  current: {}", self.current)?;
        writeln!(f, "  end: {}", self.end)?;
        writeln!(f, "  total mem: {}", self.total_mem)?;
        writeln!(f, "  max block size: {}", self.max_block_size)?;
        writeln!(f, "  sized block count: {}", self.sized_block_count)?;
        for i in 0..self.sized_block_count {
            writeln!(
                f,
                "  bin#{} size={} = {:#?}",
                i,
                Allocator::size_of_block(i),
                self.sized_blocks[i]
            )?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
