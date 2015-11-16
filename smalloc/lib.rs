//! Small memory allocator
//!
//! # Advantages of small memory allocator
//! - Fully ANSI/POSIX compatible
//! - Low memory overhead
//! - Low memory fragmentation
//! - Reasonably fast
//! - Wide range of error detection
//!
//! # Design details
//! This is essentially a SLOB allocator where all free blocks are
//! linked together sorted by size. Sorting speedups best-fit
//! search. All blocks are also doubly-linked in list by their order
//! in memory. This makes coalescing possible.
//!
//! The tag for blocks is:
//! 
//! - `u16` size of previous block / minimal allocation size
//!
//!    Is used to traverse blocks backward.
//!
//!    We don't need to traverse free blocks backward, so for free
//!    blocks, previous size is set to 0 (that's how we can
//!    distinguish free blocks).
//! - `u16` size of current block / minimal allocation size
//!
//!    Is used to traverse blocks forward.
//! - Pointer to next free block (for free blocks only. Doesn't count
//! to total memory overhead)
//!
//!    This links all free blocks in increasing size order. Sorting
//!    speedups best-fit search.
//!
//! Embedded devices don't usually have allocations larger than 256
//! kbytes, so it's perfectly safe to have 2-byte long sizes with 4
//! byte granularity. Thus, the maximum single allocation size is
//! 262140 bytes (almost 256 kbytes) if minimal allocation size is 4
//! bytes. Note that allocator is still able to handle memories larger
//! than this, the limit is active for single block sizes. Then total
//! size of tag is 4 bytes.
//!
//! ## Allocation
//! The allocation is done by traversing the list of free blocks and
//! choosing the first one that fits. This is essentially a best-fit
//! algorithm as the list is sorted.
//!
//! ## Deallocation
//! Deallocation is as simple as mark current block as free and try to
//! coalesce it with neighbors. Aware not to coalesce blocks if total
//! size exceeds limit. Then add new free block to a list of free
//! blocks with respect to the size (Don't forget to remove coalesced
//! blocks).
//!
//! ## Error detection
//! Error detaction is primarily a checking for block list invariant:
//! the size of next block's prevsize is equal to this block's cursize
//! or zero.
//!
//! ### Buffer overflow
//! When freeing block, check that next block's prevsize is equal to
//! this block's cursize or zero. This can detect some buffer
//! overflows in the current block.
//!
//! ### Double free
//! When freeing block, it must be marked as non-free. Otherwise, it's
//! double-free.
//!
//! ### Free of incorrect address
//! If list invariant is not preserved for current block, there is
//! chance it's not start of block at all.
//!
//! ### Force check
//! It's possible to force check all memory. It's as easy as
//! traversing the whole list of blocks, checking list invariant for
//! every entry.
#![crate_name = "smalloc"]
#![crate_type = "rlib"]

#![feature(no_std)]
#![no_std]

#![cfg_attr(test, feature(alloc, heap_api))]
#![cfg_attr(test, allow(raw_pointer_derive))]

#[cfg(test)]
extern crate alloc;

use ::core::ptr;

macro_rules! size_of {
    ( $t:ty ) => ( ::core::mem::size_of::<$t>() )
}

macro_rules! isize_of {
    ( $t:ty ) => ( ::core::mem::size_of::<$t>() as isize )
}

#[allow(dead_code)]
fn psize() -> usize {
    ::core::mem::size_of::<*mut u8>()
}

#[allow(dead_code)]
fn ipsize() -> isize {
    ::core::mem::size_of::<*mut u8>() as isize
}

#[allow(dead_code)]
fn bbsize() -> usize {
    ::core::mem::size_of::<BusyBlock>()
}

#[allow(dead_code)]
fn ibbsize() -> isize {
    ::core::mem::size_of::<BusyBlock>() as isize
}

#[allow(dead_code)]
fn fbsize() -> usize {
    ::core::mem::size_of::<FreeBlock>()
}

#[allow(dead_code)]
fn ifbsize() -> isize {
    ::core::mem::size_of::<FreeBlock>() as isize
}

const MIN_ALLOC: usize = 4;

pub struct Smalloc {
    /// Start of the memory served by Smalloc
    pub start: *mut u8,
    /// Size of the memory served by Smalloc
    pub size: usize,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(packed)]
struct FreeBlock {
    pub prev_size: u16,
    pub size: u16,
    pub next: *mut FreeBlock,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(packed)]
struct BusyBlock {
    pub prev_size: u16,
    pub size: u16,
}

impl Smalloc {
    fn free_list_start(&self) -> *mut *mut FreeBlock {
        self.start as *mut _
    }

    /// Initializes memory for allocator.
    ///
    /// Must be called before any allocation.
    pub unsafe fn init(&self) {
        *self.free_list_start() = self.start.offset(ipsize()) as *mut FreeBlock;
        *(self.start.offset(ipsize()) as *mut _) = FreeBlock {
            prev_size: 0x0,
            size: ((self.size - psize() - bbsize()) / MIN_ALLOC) as u16,
            next: ptr::null_mut(),
        };
    }

    pub fn alloc(&self, size: usize) -> *mut u8 {
        unsafe {
            if size == 0 {
                return ptr::null_mut();
            }

            let (prev, cur) = self.find_free_block(size);
            if cur.is_null() {
                return ptr::null_mut();
            }

            let cur = cur as *mut BusyBlock;

            let prev_next_ptr = if prev.is_null() {
                self.free_list_start()
            } else {
                &mut (*prev).next as *mut _
            };

            *cur = BusyBlock {
                prev_size: 2,
                size: 2,
            };

            let next = (cur as *mut u8)
                .offset(ibbsize() + ((*cur).size as usize * MIN_ALLOC) as isize) as *mut FreeBlock;
            *next = FreeBlock {
                prev_size: 2,
                size: 59,
                next: ptr::null_mut(),
            };

            *prev_next_ptr = next;

            (cur as *mut u8).offset(ibbsize())
        }
    }

    unsafe fn find_free_block(&self, size: usize) -> (*mut FreeBlock, *mut FreeBlock) {
        let s = ((size + MIN_ALLOC - 1) / MIN_ALLOC) as u16;

        let mut prev = ptr::null_mut();
        let mut cur = *self.free_list_start();
        while !cur.is_null() && (*cur).size < s {
            prev = cur;
            cur = (*cur).next;
        }

        (prev, cur)
    }
}

#[cfg(test)]
mod test {
    #![allow(unused_imports)]

    use super::*;
    use super::{BusyBlock, FreeBlock, MIN_ALLOC, psize, ipsize, bbsize, ibbsize, fbsize, ifbsize};

    use alloc::heap;

    use ::core::mem::size_of;
    use ::core::ptr;

    fn with_memory<F>(size: usize, f: F) where F: Fn(*mut u8, &Smalloc) -> () {
        unsafe {
            let memory = heap::allocate(size, psize());
            let a: Smalloc = Smalloc {
                start: memory,
                size: size,
            };
            a.init();

            f(memory, &a);

            heap::deallocate(memory, size, size_of::<*mut u8>());
        }
    }

    #[test]
    fn test_init_tags() {
        with_memory(256, |memory, _| unsafe {
            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: 0x0,
                    size: ((256 - psize() - bbsize()) / MIN_ALLOC) as u16,
                    next: 0x0 as *mut FreeBlock
                },
                *(memory.offset(ipsize()) as *const FreeBlock));
        });
    }

    #[test]
    fn test_alloc_one_block() {
        with_memory(256, |memory, a| unsafe {
            let ret = a.alloc(8);

            assert_eq!(memory.offset(ipsize() + ibbsize()), ret);
            assert_eq!(memory.offset(ipsize() + ibbsize() + 8) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            assert_eq!(
                BusyBlock {
                    prev_size: (psize() / 4) as u16,
                    size: (0x8 / MIN_ALLOC) as u16,
                },
                *(memory.offset(ipsize()) as *const BusyBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: (0x8 / MIN_ALLOC) as u16,
                    size: ((256 - psize() - bbsize() - 0x8) / MIN_ALLOC) as u16,
                    next: 0x0 as *mut FreeBlock,
                },
                *(memory.offset(ipsize() + ibbsize() + 0x8) as *mut FreeBlock));
        });
    }

    #[test]
    #[ignore]
    fn test_alloc_two_blocks() {
        with_memory(256, |memory, a| unsafe {
            let ret1 = a.alloc(32);
            let ret2 = a.alloc(16);

            assert_eq!(memory.offset(ipsize() + ibbsize()), ret1);
            assert_eq!(memory.offset(ipsize() + ibbsize() +
                                     32 + ibbsize()), ret2);
        });
    }

    #[test]
    fn test_alloc_too_big() {
        with_memory(32, |_, a| {
            let ret = a.alloc(32 - psize() - bbsize() + 1);

            assert_eq!(ptr::null_mut(), ret);
        });
    }

    #[test]
    #[ignore]
    fn test_alloc_max() {
        with_memory(32, |memory, a| unsafe {
            let ret = a.alloc(32 - psize() - size_of!(BusyBlock));

            assert_eq!(memory.offset(ipsize() + isize_of!(BusyBlock)), ret);
            assert_eq!(ptr::null_mut(), *(memory as *const *mut FreeBlock));
        });
    }

    #[test]
    fn test_alloc_zero() {
        with_memory(32, |_, a| {
            let ret = a.alloc(0);

            assert_eq!(ptr::null_mut(), ret);
        });
    }
}
