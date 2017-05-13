//! Small memory allocator
//!
//! *Warning: allocator has issues on memory sizes bigger than 64 Kb*
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
//!    The lowest bit is used to distinguish free/busy blocks. 0 is
//!    for busy block, 1 for free.
//! - `u16` size of current block / minimal allocation size
//!
//!    Is used to traverse blocks forward.
//! - Pointer to next free block (for free blocks only. Doesn't count
//! to total memory overhead)
//!
//!    This links all free blocks in increasing size order. Sorting
//!    speedups best-fit search.
//!
//! Embedded devices don't usually have allocations larger than 64
//! kbytes, so it's perfectly safe to have 2-byte long sizes. Thus,
//! the maximum single allocation size is 65535 bytes (almost 64
//! kbytes). Note that allocator is still able to handle memories
//! larger than this, the limit is active for single block sizes. Then
//! total size of tag is 4 bytes.
//!
//! Note: it's possible to allocate whole 65536 bytes (full 64 kbytes
//! if treat size 0 as 64 kbytes)
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

#![cfg_attr(not(test), no_std)]

#![cfg_attr(test, feature(alloc, heap_api, rand))]

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate core;
#[cfg(test)]
extern crate rand;

use ::core::ptr;

macro_rules! size_of {
    ( $t:ty ) => ( ::core::mem::size_of::<$t>() )
}

macro_rules! isize_of {
    ( $t:ty ) => ( ::core::mem::size_of::<$t>() as isize )
}

fn psize() -> usize {
    ::core::mem::size_of::<*mut u8>()
}

#[allow(cast_possible_wrap)]
fn ipsize() -> isize {
    ::core::mem::size_of::<*mut u8>() as isize
}

fn bbsize() -> usize {
    ::core::mem::size_of::<BusyBlock>()
}

#[allow(cast_possible_wrap)]
fn ibbsize() -> isize {
    ::core::mem::size_of::<BusyBlock>() as isize
}

#[allow(dead_code)]
fn fbsize() -> usize {
    ::core::mem::size_of::<FreeBlock>()
}

#[allow(cast_possible_wrap)]
fn ifbsize() -> isize {
    ::core::mem::size_of::<FreeBlock>() as isize
}

#[derive(Debug)]
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

impl FreeBlock {
    pub fn is_free(&self) -> bool {
        self.prev_size & 0x1 != 0
    }
}

impl BusyBlock {
    // It's here for symmetry
    #[allow(dead_code)]
    pub fn is_free(&self) -> bool {
        self.prev_size & 0x1 != 0
    }
}

#[cfg(test)]
impl ::core::fmt::Display for FreeBlock {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        f.debug_struct("FreeBlock")
            .field("prev_size", &self.prev_size)
            .field("size", &self.size)
            .field("next", &self.next)
            .finish()
    }
}

#[cfg(test)]
impl ::core::fmt::Display for BusyBlock {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        f.debug_struct("BusyBlock")
            .field("prev_size", &self.prev_size)
            .field("size", &self.size)
            .finish()
    }
}

const MAX_ALLOC: usize = 64*1024 - 4;

impl Smalloc {
    fn free_list_start(&self) -> *mut *mut FreeBlock {
        self.start as *mut _
    }

    /// Initializes memory for allocator.
    ///
    /// Must be called before any allocation.
    #[allow(cast_possible_truncation)] // cur_size is guaranteed to be less than MAX_ALLOC
    #[allow(cast_possible_wrap)] // MAX_ALLOC should not wrap when cast to isize
    pub unsafe fn init(&self) {
        *self.free_list_start() = self.start.offset(ipsize()) as *mut FreeBlock;

        let mut prev_size = 0;
        let mut cur_offset = ipsize();
        let mut size = self.size - psize();
        while size != 0 {
            let cur_size = ::core::cmp::min(MAX_ALLOC, size - bbsize());
            size -= cur_size + bbsize();
            *(self.start.offset(cur_offset) as *mut _) = FreeBlock {
                prev_size: prev_size + 1,
                size: cur_size as u16,
                next: if size == 0 { ptr::null_mut() } else { self.start.offset(cur_offset + ibbsize() + MAX_ALLOC as isize) as *mut _ },
            };

            prev_size = cur_size as u16;
            cur_offset += cur_size as isize + ibbsize();
        }
    }

    #[allow(cast_possible_truncation)] // size is checked to be u16
    #[allow(cast_possible_wrap)]
    pub unsafe fn alloc(&self, mut size: usize) -> *mut u8 {
        if size == 0 {
            return ptr::null_mut();
        }
        if size > ::core::u16::MAX as usize {
            return ptr::null_mut();
        }

        size = (size + psize() - 1) & !(psize() - 1);

        let (prev_empty, cur) = self.find_free_block(size as u16);
        if cur.is_null() {
            return ptr::null_mut();
        }

        // remove block from free list
        *self.get_next_ptr(prev_empty) = (*cur).next;

        let prev_cur_size = (*cur).size;
        if (prev_cur_size as isize) - (size as isize) < ifbsize() {
            size = prev_cur_size as usize;
        } else {
            let split_next = (cur as *mut u8)
                .offset(ibbsize() + size as isize) as *mut FreeBlock;
            *split_next = FreeBlock {
                prev_size: (size + 1) as u16,
                size: prev_cur_size - size as u16 - bbsize() as u16,
                next: ptr::null_mut(),
            };

            let split_next_next = (split_next as *mut u8).offset((*split_next).size as isize + ibbsize()) as *mut FreeBlock;
            if split_next_next < self.start.offset(self.size as isize) as *mut FreeBlock {
                (*split_next_next).prev_size = (*split_next).size + (*split_next_next).is_free() as u16;
            }

            self.install_free_block(split_next);
        }

        let cur = cur as *mut BusyBlock;

        *cur = BusyBlock {
            prev_size: (*cur).prev_size - 1,
            size: size as u16,
        };

        (cur as *mut u8).offset(ibbsize())
    }

    #[allow(cast_possible_wrap)]
    #[allow(cast_possible_truncation)] // bbsize < u16
    pub unsafe fn free(&self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        let mut block = ptr.offset(-ibbsize()) as *mut FreeBlock;

        // try merge with previous
        let prev_block = (block as *mut u8).offset(-((*block).prev_size as isize) - ibbsize()) as *mut FreeBlock;
        let next_block = (block as *mut u8).offset(ibbsize() + (*block).size as isize) as *mut FreeBlock;

        if (*block).prev_size != 0 && (*prev_block).is_free() &&
            (*block).size as usize + (*prev_block).size as usize + bbsize() < MAX_ALLOC
        {
            let prev = self.find_previous_block(prev_block);
            // remove prev_block from list temporary
            *self.get_next_ptr(prev) = (*prev_block).next;

            if (next_block as *mut u8) < self.start.offset(self.size as isize) {
                (*next_block).prev_size += (*prev_block).size + bbsize() as u16;
            }
            (*prev_block).size += (*block).size + bbsize() as u16;
            block = prev_block;
        } else {
            // mark block as free
            (*block).prev_size += 1;
        }

        // try merge with next
        if (next_block as *mut u8) < self.start.offset(self.size as isize) &&
            (*next_block).is_free() &&
            (*block).size as usize + (*next_block).size as usize + bbsize() < MAX_ALLOC {
                let prev = self.find_previous_block(next_block);
                *self.get_next_ptr(prev) = (*next_block).next;

                let next_next = (next_block as *mut u8).offset(ibbsize() + (*next_block).size as isize) as *mut FreeBlock;
                if (next_next as *mut u8) < self.start.offset(self.size as isize) {
                    (*next_next).prev_size += (*block).size + bbsize() as u16;
                }

                (*block).size += bbsize() as u16 + (*next_block).size;
            }

        self.install_free_block(block);
    }

    unsafe fn find_free_block(&self, size: u16) -> (*mut FreeBlock, *mut FreeBlock) {
        self.find_free_after(size, ptr::null_mut())
    }

    unsafe fn find_free_after(&self, size: u16, after: *mut FreeBlock) -> (*mut FreeBlock, *mut FreeBlock) {
        let mut prev = after;
        let mut cur = *self.get_next_ptr(prev);

        while !cur.is_null() && (*cur).size < size {
            prev = cur;
            cur = (*cur).next;
        }

        (prev, cur)
    }

    unsafe fn install_free_block(&self, block: *mut FreeBlock) {
        let (mut prev, mut next) = self.find_free_block((*block).size);

        // sort them by memory address when the size is same.
        // That allows one neat optimization in the free.
        while !next.is_null() && (*next).size == (*block).size && block > next {
            prev = next;
            next = (*next).next;
        }

        *self.get_next_ptr(prev) = block;
        (*block).next = next;
    }

    unsafe fn find_previous_block(&self, block: *mut FreeBlock) -> *mut FreeBlock {
        let mut prev = ptr::null_mut();
        let mut cur = *self.free_list_start();

        while cur != block {
            prev = cur;
            cur = (*cur).next;
        }

        prev
    }

    unsafe fn get_next_ptr(&self, block: *mut FreeBlock) -> *mut *mut FreeBlock {
        if block.is_null() {
            self.free_list_start()
        } else {
            &mut (*block).next as *mut _
        }
    }

    #[cfg(test)]
    unsafe fn debug_print(&self) {
        // print free list
        println!("Free list:");
        let mut cur = *self.free_list_start();
        while !cur.is_null() {
            println!("{:p}", cur);
            cur = (*cur).next;
        }

        // print block list
        let mut block = self.start.offset(ipsize()) as *const FreeBlock;
        while block < self.start.offset(self.size as isize) as *const _ {
            if (*block).is_free() {
                println!("{:p}: {}", block, *block);
            } else {
                println!("{:p}: {}", block, *(block as *const BusyBlock));
            }

            block = (block as *const u8).offset((*block).size as isize + ibbsize()) as *const _;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use super::{BusyBlock, FreeBlock, psize, ipsize, bbsize, ibbsize, fbsize, ifbsize};

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
                    prev_size: 0x1,
                    size: (256 - psize() - bbsize()) as u16,
                    next: 0x0 as *mut FreeBlock
                },
                *(memory.offset(ipsize()) as *const FreeBlock));
        });
    }

    #[test]
    fn test_init_too_big() {
        with_memory(130 * 1024, |memory, _| unsafe {
            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: 0x1,
                    size: (64*1024 - 4) as u16,
                    next: memory.offset(ipsize() + ibbsize() + 64*1024 - 4) as *mut FreeBlock,
                },
                *(memory.offset(ipsize()) as *const FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: (64*1024 - 4 + 1) as u16,
                    size: (64*1024 - 4) as u16,
                    next: memory.offset(ipsize() + 2*ibbsize() + 2*(64*1024 - 4)) as *mut FreeBlock,
                },
                *(memory.offset(ipsize() + ibbsize() + 64*1024 - 4) as *mut FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: (64*1024 - 4 + 1) as u16,
                    size: (130*1024 - psize() - 3*bbsize() - 2*(64*1024 - 4)) as u16,
                    next: ptr::null_mut(),
                },
                *(memory.offset(ipsize() + 2*ibbsize() + 2*(64*1024 - 4)) as *mut FreeBlock));
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
                    prev_size: 0,
                    size: 8,
                },
                *(memory.offset(ipsize()) as *const BusyBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: 0x9,
                    size: (256 - psize() - bbsize() - 0x8 - bbsize()) as u16,
                    next: 0x0 as *mut FreeBlock,
                },
                *(memory.offset(ipsize() + ibbsize() + 0x8) as *mut FreeBlock));
        });
    }

    #[test]
    fn test_alloc_two_blocks() {
        with_memory(256, |memory, a| unsafe {
            let ret1 = a.alloc(32);
            let ret2 = a.alloc(16);

            // memory layout after test:
            // - pointer to free block
            assert_eq!(*(memory.offset(0) as *const *const FreeBlock),
                       memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16) as *const _);
            // - busy block for 32 bytes
            assert_eq!(
                BusyBlock {
                    prev_size: 0,
                    size: 32,
                },
                *(memory.offset(ipsize()) as *const BusyBlock));
            // - 32 bytes of data
            assert_eq!(memory.offset(ipsize() + ibbsize()), ret1);
            // - busy block for 16 bytes
            assert_eq!(
                BusyBlock {
                    prev_size: 32,
                    size: 16,
                },
                *(memory.offset(ipsize() + ibbsize() + 32) as *const BusyBlock));
            // - 16 bytes of data
            assert_eq!(memory.offset(ipsize() + ibbsize() + 32 + ibbsize()),
                       ret2);
            // - free block till end
            assert_eq!(
                FreeBlock {
                    prev_size: 17,
                    size: (256 - psize() - bbsize() - 32 - bbsize() - 16 - bbsize()) as u16,
                    next: 0x0 as *mut _,
                },
                *(memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16) as *const FreeBlock));
        });
    }

    #[test]
    fn test_alloc_too_big() {
        with_memory(36 + psize(), |_, a| unsafe {
            let ret = a.alloc(32 + 1);

            assert_eq!(ptr::null_mut(), ret);
        });
    }

    #[test]
    fn test_alloc_max() {
        with_memory(36 + psize(), |memory, a| unsafe {
            let ret = a.alloc(32);

            assert_eq!(memory.offset(ipsize() + ibbsize()), ret);
            assert_eq!(ptr::null_mut(), *(memory as *const *mut FreeBlock));
        });
    }

    #[test]
    fn test_alloc_zero() {
        with_memory(32, |_, a| unsafe {
            let ret = a.alloc(0);

            assert_eq!(ptr::null_mut(), ret);
        });
    }

    #[test]
    fn test_free_single_block() {
        with_memory(256, |memory, a| unsafe {
            let ptr = a.alloc(32);
            a.free(ptr);

            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: 0x1,
                    size: (256 - psize() - bbsize()) as u16,
                    next: 0x0 as *mut FreeBlock
                },
                *(memory.offset(ipsize()) as *const FreeBlock));
        });
    }

    #[test]
    fn test_free_first() {
        // This test is similar to test_alloc_two_blocks
        with_memory(256, |memory, a| unsafe {
            let ptr1 = a.alloc(32);
            let _ptr2 = a.alloc(16);
            a.free(ptr1);

            // The memory now is:
            // - pointer to new free block
            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                      *(memory as *const *mut FreeBlock));

            // - free block itself (points to next big block)
            assert_eq!(
                FreeBlock {
                    prev_size: 1,
                    size: 32,
                    next: memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16) as *mut _,
                },
                *(memory.offset(ipsize()) as *const FreeBlock));

            // - second busy block
            assert_eq!(
                BusyBlock {
                    prev_size: 32,
                    size: 16,
                },
                *(memory.offset(ipsize() + ibbsize() + 32) as *const BusyBlock));

            // - free block till the end
            assert_eq!(
                FreeBlock {
                    prev_size: 17,
                    size: (256 - psize() - bbsize() - 32 - bbsize() - 16 - bbsize()) as u16,
                    next: 0x0 as *mut _,
                },
                *(memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16) as *const FreeBlock));
        });
    }

    #[test]
    fn test_free_merge_before_next() {
        with_memory(512, |memory, a| unsafe {
            let ptr1 = a.alloc(32);
            let ptr2 = a.alloc(16);
            let _ptr3 = a.alloc(16);

            a.free(ptr2);
            a.free(ptr1);

            // The memory now is:
            // - pointer to free block at start
            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));

            // - merged free block
            assert_eq!(
                FreeBlock {
                    prev_size: 1,
                    size: 32 + bbsize() as u16 + 16,
                    next: memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16 + ibbsize() + 16) as *mut _,
                },
                *(memory.offset(ipsize()) as *const FreeBlock));

            // - busy block
            assert_eq!(
                BusyBlock {
                    prev_size: 32 + bbsize() as u16 + 16,
                    size: 16,
                },
                *(memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16) as *const _));

            // - free block till the end
            assert_eq!(
                FreeBlock {
                    prev_size: 17,
                    size: 512 - (psize() + bbsize() + 32 + bbsize() + 16 + bbsize() + 16 + bbsize()) as u16,
                    next: ptr::null_mut(),
                },
                *(memory.offset(ipsize() + ibbsize() + 32 + ibbsize() + 16 + ibbsize() + 16) as *const _));
        });
    }

    #[test]
    fn test_free_merge_previous() {
        with_memory(512, |memory, a| unsafe {
            let _ptr1 = a.alloc(16);
            let ptr2 = a.alloc(32);
            let ptr3 = a.alloc(16);
            let _ptr4 = a.alloc(32);

            a.free(ptr2);
            a.free(ptr3);

            // The memory now is:
            // - pointer to free block near start
            assert_eq!(memory.offset(ipsize() + ibbsize() + 16) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));

            // - busy block
            assert_eq!(
                BusyBlock {
                    prev_size: 0,
                    size: 16,
                },
                *(memory.offset(ipsize()) as *mut BusyBlock));

            // - free block
            assert_eq!(
                FreeBlock {
                    prev_size: 17,
                    size: 32 + bbsize() as u16 + 16,
                    next: memory.offset(ipsize() + 4*ibbsize() + 16 + 32 + 16 + 32) as *mut _,
                },
                *(memory.offset(ipsize() + ibbsize() + 16) as *mut _));

            // - busy block
            assert_eq!(
                BusyBlock {
                    prev_size: 32 + bbsize() as u16 + 16,
                    size: 32,
                },
                *(memory.offset(ipsize() + 3*ibbsize() + 16 + 32 + 16) as *mut _));

            // - free block
            assert_eq!(
                FreeBlock {
                    prev_size: 33,
                    size: 512 - (psize() + 5*bbsize() + 16 + 32 + 16 + 32) as u16,
                    next: ptr::null_mut(),
                },
                *(memory.offset(ipsize() + 4*ibbsize() + 16 + 32 + 16 + 32) as *mut _));
        });
    }

    #[test]
    fn test_free_merge_both() {
        with_memory(512, |memory, a| unsafe {
            let _ptr1 = a.alloc(16);
            let ptr2 = a.alloc(32);
            let ptr3 = a.alloc(16);
            let ptr4 = a.alloc(24);
            let _ptr5 = a.alloc(32);

            a.free(ptr2);
            a.free(ptr4);
            a.free(ptr3);

            // The memory now is:
            // - pointer to free block near start
            assert_eq!(memory.offset(ipsize() + ibbsize() + 16) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));

            // - busy block (_ptr1)
            assert_eq!(
                BusyBlock {
                    prev_size: 0,
                    size: 16,
                },
                *(memory.offset(ipsize()) as *mut BusyBlock));

            // - free block (ptr2, ptr3, ptr4 merged)
            assert_eq!(
                FreeBlock {
                    prev_size: 17,
                    size: 32 + bbsize() as u16 + 16 + bbsize() as u16 + 24,
                    next: memory.offset(ipsize() + 5*ibbsize() + 16 + 32 + 16 + 24 + 32) as *mut _,
                },
                *(memory.offset(ipsize() + ibbsize() + 16) as *mut _));

            // - busy block (_ptr5)
            assert_eq!(
                BusyBlock {
                    prev_size: 32 + bbsize() as u16 + 16 + bbsize() as u16 + 24,
                    size: 32,
                },
                *(memory.offset(ipsize() + 4*ibbsize() + 16 + 32 + 16 + 24) as *mut _));

            // - free block till end
            assert_eq!(
                FreeBlock {
                    prev_size: 33,
                    size: 512 - (psize() + 6*bbsize() + 16 + 32 + 16 + 24 + 32) as u16,
                    next: ptr::null_mut(),
                },
                *(memory.offset(ipsize() + 5*ibbsize() + 16 + 32 + 16 + 24 + 32) as *mut _));
        });
    }

    #[test]
    fn test_free_all() {
        with_memory(512, |memory, a| unsafe {
            let ptr1 = a.alloc(16);
            let ptr2 = a.alloc(8);
            let ptr3 = a.alloc(128);
            let ptr4 = a.alloc(32);
            let ptr5 = a.alloc(24);

            a.free(ptr4);
            a.free(ptr5);
            a.free(ptr1);
            a.free(ptr3);
            a.free(ptr2);

            assert_eq!(memory.offset(ipsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            assert_eq!(
                FreeBlock {
                    prev_size: 0x1,
                    size: (512 - psize() - bbsize()) as u16,
                    next: 0x0 as *mut FreeBlock
                },
                *(memory.offset(ipsize()) as *const FreeBlock));
        });
    }

    #[test]
    fn test_free_list_is_sorted() {
        with_memory(512, |memory, a| unsafe {
            let ptr1 = a.alloc(16);
            let _ptr2 = a.alloc(8);
            let ptr3 = a.alloc(16);
            let _ptr4 = a.alloc(8);
            let ptr5 = a.alloc(16);
            let _ptr6 = a.alloc(8);

            a.free(ptr1);
            a.free(ptr5);
            a.free(ptr3);

            // free list now is:
            // - start -> ptr1
            assert_eq!(ptr1.offset(-ibbsize()) as *mut FreeBlock,
                       *(memory as *const *mut FreeBlock));
            // - ptr1 -> ptr3
            assert_eq!(ptr3.offset(-ibbsize()) as *mut FreeBlock,
                       *(ptr1 as *const *mut FreeBlock));
            // - ptr3 -> ptr5
            assert_eq!(ptr5.offset(-ibbsize()) as *mut FreeBlock,
                       *(ptr3 as *const *mut FreeBlock));
            // - ptr5 -> rest
            assert_eq!(memory.offset(ipsize() + 6*ibbsize() + 3*16 + 3*8) as *mut FreeBlock,
                       *(ptr5 as *const *mut FreeBlock));
        });
    }

    #[test]
    fn test_alloc_align() {
        fn round_up(value: u16) -> u16 {
            (value + psize() as u16 - 1) / psize() as u16 * psize() as u16
        }

        with_memory(512, |_, a| unsafe {
            let ptr1 = a.alloc(1);
            let ptr2 = a.alloc(14);
            let ptr3 = a.alloc(17);

            // allocation granularity is size of pointer (it should be
            // larger than size of BusyBlock)
            assert_eq!(round_up(1),  (*(ptr1.offset(-ibbsize()) as *const BusyBlock)).size);
            assert_eq!(round_up(14), (*(ptr2.offset(-ibbsize()) as *const BusyBlock)).size);
            assert_eq!(round_up(17), (*(ptr3.offset(-ibbsize()) as *const BusyBlock)).size);
        });
    }

    #[test]
    fn test_dont_split_too_small() {
        with_memory(512, |memory, a| unsafe {
            let ptr1 = a.alloc(32);
            let _ptr2 = a.alloc(8);
            a.free(ptr1);
            let ptr3 = a.alloc(32 - psize());

            // block of ptr1 should be reused
            assert_eq!(ptr1, ptr3);

            // The memory is:
            // - pointer to only free block in memory
            assert_eq!(memory.offset(ipsize() + 2*ibbsize() + 32 + 8) as *mut FreeBlock,
                       *(memory as *mut _));
            // - BusyBlock as for ptr1
            assert_eq!(
                BusyBlock {
                    prev_size: 0,
                    size: 32,
                },
                *(memory.offset(ipsize()) as *mut _));
            // - BusyBlock for ptr3
            assert_eq!(
                BusyBlock {
                    prev_size: 32,
                    size: 8,
                },
                *(memory.offset(ipsize() + ibbsize() + 32) as *mut _));
            // - free block till end
            assert_eq!(
                FreeBlock {
                    prev_size: 9,
                    size: 512 - (psize() + 3*bbsize() + 32 + 8) as u16,
                    next: ptr::null_mut(),
                },
                *(memory.offset(ipsize() + 2*ibbsize() + 32 + 8) as *mut _));
        });
    }

    #[test]
    fn test_dont_merge_too_big_with_next() {
        with_memory(168*1024, |_memory, a| unsafe {
            let ptr1 = a.alloc(48*1024);
            let ptr2 = a.alloc(48*1024);
            let _ptr3 = a.alloc(48*1024);

            a.free(ptr2);
            a.free(ptr1);

            // TODO
        });
    }

    #[test]
    fn test_dont_merge_too_big_with_prev() {
        with_memory(256*1024, |_memory, a| unsafe {
            let ptr1 = a.alloc(48*1024);
            let ptr2 = a.alloc(48*1024);
            let _ptr3 = a.alloc(48*1024);

            a.free(ptr1);
            a.free(ptr2);

            // TODO
        });
    }

    #[test]
    fn test_free_null() {
        with_memory(256, |_, a| unsafe {
            a.free(ptr::null_mut());
        });
    }

    #[test]
    fn test_endurance() {
        // That's a fucking trick because standard rand doesn't export
        // StdRng for unknown reason
        #[cfg(target_pointer_width = "32")]
        use ::rand::IsaacRng as IsaacWordRng;
        #[cfg(target_pointer_width = "64")]
        use ::rand::Isaac64Rng as IsaacWordRng;

        use ::rand::{SeedableRng, Rng};
        use ::std::vec::Vec;
        use ::std::intrinsics::write_bytes;

        const MEMORY_SIZE: usize = 64*1024; // 64*1024;

        with_memory(MEMORY_SIZE, |_, a| unsafe {
            // Reproducability is a must
            let mut rng = IsaacWordRng::from_seed(&[42,42,42]);

            let mut allocs = Vec::new();

            for _ in 1..10000 {
                if allocs.is_empty() || rng.gen() {
                    let size = rng.gen::<usize>() % MEMORY_SIZE;
                    let ptr = a.alloc(size);

                    if !ptr.is_null() {
                        println!("alloc {} = {:p}", size, ptr);
                        write_bytes(ptr, 0x00, size);
                        allocs.push(ptr);
                    }
                } else {
                    let i = rng.gen_range(0, allocs.len());
                    let ptr = allocs.remove(i);

                    println!("free {:p}", ptr);

                    a.free(ptr);
                }
            }
        });
    }

    #[test]
    fn test_update_split_next_prev() {
        const MEMORY_SIZE: usize = 64*1024;

        with_memory(MEMORY_SIZE, |_, a| unsafe {
            a.debug_print();

            let ptr1 = a.alloc(36239);
            println!("ptr1 = {:p}", ptr1);
            a.debug_print();
            println!("");

            let ptr2 = a.alloc(20000);
            println!("ptr2 = {:p}", ptr2);
            a.debug_print();
            println!("");

            println!("free ptr1 {:p}", ptr1);
            a.free(ptr1);
            a.debug_print();
            println!("");

            let ptr3 = a.alloc(32768);
            println!("ptr3 = {:p}", ptr3);
            a.debug_print();
            println!("");

            println!("free ptr3 {:p}", ptr3);
            a.free(ptr3);
            a.debug_print();
            println!("");
        });
    }
}
