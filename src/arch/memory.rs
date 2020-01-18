// Author: Moritz Doll
// License: GPLv3

//use core::{fmt, ops::RangeInclusive};

use core::ops;
//use core::iter;
//use core::slice;
use armv7::structures::paging;
use armv7::PhysicalAddress;
use crate::bsp::memory_map;
use crate::kernel::kernel_info;

#[derive(Copy,Clone,Debug)]
pub enum MemoryError {
    PagingError(paging::PageError),
    OutOfMemory,
}

pub type Result<T> = ::core::result::Result<T,MemoryError>;

#[derive(Copy,Clone,Debug)]
pub enum MemoryType {
    DRAM,
    Device,
}

#[derive(Copy,Clone,Debug)]
pub enum Permission {
    ReadOnly,
    ReadWrite,
}


#[derive(Copy,Clone,Debug)]
pub struct PageAttributes {
    mem_type: MemoryType,
    perms: Permission,
    unpriv: bool,
    accessed: bool,
    dirty: bool,
    exec: bool,
}

impl Default for PageAttributes {
    fn default() -> PageAttributes {
        PageAttributes {
            mem_type: MemoryType::DRAM,
            perms: Permission::ReadWrite,
            unpriv: true,
            accessed: false,
            dirty: false,
            exec: false,
        }
    }
}

/// Attributes for physical frames
#[derive(Copy,Clone,Debug)]
pub struct FrameAttributes(u8); // Size will be reduced

impl FrameAttributes {
    pub fn new_inactive() -> FrameAttributes {
        FrameAttributes(0b0)
    }

    pub fn new_active() -> FrameAttributes {
        FrameAttributes(0b1)
    }

    pub fn new_kernel() -> FrameAttributes {
        FrameAttributes(0b11)
    }
    // Todo: Bits for Access and Dirty
}

pub struct PageTableManagement {
    address: PhysicalAddress,
    num_free: u8,
}



// Physical Memory Dummy allocator
pub struct PhysicalMemory {
    size: u32,
}

impl PhysicalMemory {
    pub fn new(size: u32) -> Self {
        PhysicalMemory { size: size }
    }
}

impl ops::Drop for PhysicalMemory {
    fn drop(&mut self) {
    }
}

pub const NUM_BITMAP_ENTRIES: usize = (memory_map::DRAM_SIZE_KB / (4 * 32) ) as usize;

/// Struct that tracks the availibility of physical memory
/// Each entry contains a bitmap of 32 pages = 128 Kb memory
pub struct PhysicalMemoryMap {
    table: [u32; NUM_BITMAP_ENTRIES],
    memory_start: PhysicalAddress,
}

impl PhysicalMemoryMap {
    pub unsafe fn get_entry(&mut self, entry: usize) -> u32 {
        self.table[entry]
    }
    /// This is the the totally unchecked free function - of course it is not safe to use.
    /// This function should not be public
    pub unsafe fn free_raw(&mut self, entry: usize, bits_to_free: u32) -> paging::Result<()> {
        if entry > NUM_BITMAP_ENTRIES - 1 {
            return Err(paging::PageError::NotInRange);
        }
        // We delete all marked bits from the bitmap
        self.table[entry] &= !bits_to_free;
        Ok(())
    }
    pub unsafe fn alloc_raw(&mut self, entry: usize, bits_to_alloc: u32) -> paging::Result<()> {
        if entry > NUM_BITMAP_ENTRIES - 1 {
            return Err(paging::PageError::NotInRange);
        }
        // We add all marked bits to the bitmap
        self.table[entry] |= bits_to_alloc;
        Ok(())
    }
    unsafe fn find_empty_entry(&mut self) -> Result<(usize, u32)> {
        let base_ptr = self.table.as_ptr();
        let mut iter = self.table.iter_mut().filter(|num| is_empty(**num));
        match iter.next() {
            None => Err(MemoryError::OutOfMemory),
            Some(bitmap_ptr) => {
                *bitmap_ptr = 0xffff_ffff;
                let ptr_c = bitmap_ptr as *const u32;
                let index = ptr_c.offset_from(base_ptr) as usize;
                Ok((index, *bitmap_ptr))
            }
        }
    }
    /// Try to allocate num_pages. Returns the index of the bitmap and the bitmask of the allocated
    /// pages
    pub unsafe fn find_entries(&mut self, num_pages: u32) -> Result<(usize, u32)> {
        let base_ptr = self.table.as_ptr();
        if num_pages > 32 {
            self.find_empty_entry()
        } else {
            let mut iter = self.table.iter_mut().filter(|num| is_not_full(**num) && (num.count_zeros() > num_pages));
            let reference = match iter.next() {
                None => return Err(MemoryError::OutOfMemory),
                Some(refu32) => refu32,
            };
            // Mark the correct bits
            let bitmask = match calculate_bitmask(*reference, num_pages) {
                None => return Err(MemoryError::OutOfMemory),
                Some(bits) => bits,
            };
            // remove the correct bits from the reference
            *reference |= bitmask;
            let ref_u32 = reference as *const u32;
            let index = ref_u32.offset_from(base_ptr) as usize;
            Ok((index,bitmask))
        }
    }
    
    pub fn get_physical_address(&self, index: usize, offset: u32) -> PhysicalAddress {
        if offset > 32 {
            panic!("Out of bounds!");
        }
        let offset_from_base = ((index as u32) * 32 + offset) * 4;
        self.memory_start + (offset_from_base * 1024)
    }
    pub fn allocate_frame(&mut self) -> Result<PhysicalAddress> {
        let (index, bitmask) = unsafe { self.find_entries(1) }?;
        let offset = bitmask.trailing_zeros();
        Ok(self.get_physical_address(index, offset))
    }

    fn _index(&self, item: &u32) -> Option<usize> {
        let index = (item as *const _ as usize - self.table.as_ptr() as usize) / 4;
        if NUM_BITMAP_ENTRIES > index {
            Some(index)
        } else {
            None
        }
    }

    pub unsafe fn alloc_kernel_frames(&mut self) -> Result<()> {
        let k_size = kernel_info::kernel_memory_size() as u32;
        let mut k_frames = k_size / 4096;
        let mut iter = self.table.iter_mut();
        while k_frames > 0 {
            let reference = match iter.next() {
                None => return Err(MemoryError::OutOfMemory),
                Some(refu32) => refu32,
            };
            if k_frames > 32 {
                *reference = 0xffff_ffff;
                k_frames -= 32;
            } else {
                *reference = left_shift_ones(k_frames);
                return Ok(())
            }
        }
        Ok(())
    }
}

pub fn kernel_frames() -> u32 {
    let k_size = kernel_info::kernel_memory_size() as u32;
    k_size / 4096
}

fn calculate_bitmask(bitmap: u32, num: u32) -> Option<u32> {
    if num > bitmap.count_zeros() {
        return None
    }
    for x in 0..32 {
        let shifted = bitmap << x;
        if shifted.count_zeros() - x == num {
            return Some( (!shifted) >> x)
        }
    }
    None
}

fn is_empty(num: u32) -> bool {
    num == 0
}
fn is_not_full(num: u32) -> bool {
    !(num == 0xffff_ffff)
}


// for simplicity, we first implement the function that marks the first pages as not-available (1)
// given the size of the kernel space, we calculate how many u32's we have set to full
// (0xffff_ffff) and how we have to color the final u32
fn left_shift_ones(shift: u32) -> u32 {
    !(!0 << shift)
}

fn right_shift_ones(shift: u32) -> u32 {
    !(!0 >> shift)
}


pub static mut PHYSICAL_MEMORY: PhysicalMemoryMap = PhysicalMemoryMap { table: [0; NUM_BITMAP_ENTRIES], memory_start: memory_map::DRAM_START };

// Virtual Memory Dummy allocator
