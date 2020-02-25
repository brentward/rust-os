use std::{fmt, mem, usize};
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

const BLOCK_SIZE_COUNT: usize = mem::size_of::<usize>() * 8 - 3;

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    fragmentation: usize,
    total_mem: usize,
    max_block_size: usize,
    bin_count: usize,
    current: usize,
    end: usize,
    block_bins: [LinkedList; BLOCK_SIZE_COUNT],
}
impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let current = start;
        let total_mem = end - current;
        let max_block_size = 1 << (mem::size_of::<usize>() * 8 - total_mem.leading_zeros() as usize - 1);
        let bin_count = (max_block_size as u64).trailing_zeros() as usize - 2;
        let fragmentation = 0;
        // let current = align_up(current, max_block_size);
        // let fragmentation = current - start;
        // let (max_block_size, bin_count, current) = if end - current >= max_block_size {
        //     (max_block_size, bin_count, current)
        // } else {
        //     let bin_count = bin_count - 1;
        //     let max_block_size = Allocator::size_of_bin(bin_count - 1);
        //     let current = align_up(start, max_block_size);
        //     (max_block_size, bin_count, current)
        // };
        let mut block_bins = [LinkedList::new(); BLOCK_SIZE_COUNT];
        // unsafe { block_bins[bin_count - 1].push(current as *mut usize) };
        // let current = current + max_block_size;

        Allocator {
            fragmentation,
            total_mem,
            max_block_size,
            bin_count,
            current,
            end,
            block_bins,
        }
        // let layout = &Layout::from_size_align(2usize, 8).unwrap();
        // match allocator.populate_from_above(0, layout) {
        //     Some(addr) => {
        //         unsafe { allocator.block_bins[0].push(addr) };
        //         allocator
        //     },
        //     None => panic!("Nothing was returned from populate from above"),
        // }
    }

    fn size_of_bin(index: usize) -> usize {
        2usize.pow(index as u32 + 3)
    }

    fn bin_index_for_layout(&self, layout: &Layout) -> usize {
        let required_size = layout.size().max(layout.align());
        for index in 0..self.block_bins.len() {
            if Allocator::size_of_bin(index) >= required_size{
                return index
            }
        }
        panic!("layout will cause memory address overflow");
    }

    fn split_push_return(&mut self,
                         option: Option<*mut usize>,
                         index: usize,
                         layout: &Layout) -> Option<*mut usize> {
        match option {
            Some(addr) => {
                let (low, high) = unsafe {
                    split_addr(addr, Allocator::size_of_bin(index))
                };
                let low_align = align_up(low as usize, layout.align());
                let high_align = align_up(high as usize, layout.align());
                let closest = if low_align - low as usize <= high_align - high as usize {
                    unsafe {
                        self.block_bins[index].push(high);
                        low
                    }
                } else {
                    unsafe {
                        self.block_bins[index].push(low);
                        high
                    }
                };
                Some(closest)
            }
            None => None

        }
    }

    /// Pops an item from the list containing block sises one larger than
    /// `index`, splits it in half and checks for the half closest alignment
    /// to the `layout` and returns it as `Some(*mut usize)` while pushing the other
    /// into the list at `index`. If none are above it will search up the
    /// list recursively. `None` will be returned if none of the lists have members.
    ///
    /// The effect of this that all lists between `index` and the next highest list
    /// with that is not empty will get 1 item if `Some` is returned and no change
    /// if `None is returned.
    fn populate_from_above(&mut self, index: usize, layout: &Layout) -> Option<*mut usize> {
        if index < self.bin_count {
            if self.block_bins[index + 1].is_empty() {
                let option = self.populate_from_above(index + 1, layout);
                match self.split_push_return(option, index, layout) {
                    None => None,
                    some => some,
                }
            } else {
                let option = self.block_bins[index + 1].pop();
                match self.split_push_return(option, index, layout) {
                    None => None,
                    some => some,
                }
            }
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
        let index =  self.bin_index_for_layout(&layout);
        for node in self.block_bins[index].iter_mut() {
            if has_alignment(node.value() as usize, layout.align()) {
                return Ok(node.pop() as *mut u8)
            }
        }
        let aligned_addr = align_up(self.current, Allocator::size_of_bin(index));
        match self.populate_from_above(index, &layout) {
            Some(addr) => {
                if has_alignment(addr as usize, layout.align()) {
                    Ok(addr as *mut u8)
                } else {
                    Err(AllocErr::Exhausted { request: layout })
                }
            },
            None => {
                if aligned_addr + Allocator::size_of_bin(index) > self.end {
                    Err(AllocErr::Exhausted { request: layout })
                } else {
                    self.fragmentation += aligned_addr - self.current;
                    self.current = aligned_addr + Allocator::size_of_bin(index);
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
        let index = self.bin_index_for_layout(&layout);
        unsafe {
            self.block_bins[index].push(ptr as *mut usize)
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Allocator {{")?;
        writeln!(f, "  fragmentation: {}", self.fragmentation)?;
        writeln!(f, "  current: {}", self.current)?;
        writeln!(f, "  end: {}", self.end)?;
        writeln!(f, "  unallocated mem: {}", self.end - self.current)?;
        writeln!(f, "  total mem: {}", self.total_mem)?;
        writeln!(f, "  max block size: {}", self.max_block_size)?;
        writeln!(f, "  sized block count: {}", self.bin_count)?;
        for i in 0..self.bin_count {
            writeln!(
                f,
                "  bin#{} size={} = {:#?}",
                i,
                Allocator::size_of_bin(i),
                self.block_bins[i]
            )?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
