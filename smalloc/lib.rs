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

#![feature(no_std)]
#![no_std]

#![cfg_attr(test, feature(alloc, heap_api))]

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
        const MAX_ALLOC: usize = 64*1024 - 4;

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

    pub fn alloc(&self, size: usize) -> *mut u8 {
        unsafe {
            if size == 0 {
                return ptr::null_mut();
            }

            let (prev_empty, cur) = self.find_free_block(size as u16);
            if cur.is_null() {
                return ptr::null_mut();
            }

            let prev_cur_size = (*cur).size;

            let cur = cur as *mut BusyBlock;

            let prev_next_ptr = self.get_next_ptr(prev_empty);

            *cur = BusyBlock {
                prev_size: (*cur).prev_size - 1,
                size: size as u16,
            };

            let next = (cur as *mut u8)
                .offset(ibbsize() + (*cur).size as isize) as *mut FreeBlock;
            if next < self.start.offset(self.size as isize) as *mut _ {
                *next = FreeBlock {
                    prev_size: (size + 1) as u16,
                    size: prev_cur_size - size as u16 - bbsize() as u16,
                    next: ptr::null_mut(),
                };

                *prev_next_ptr = next;
            } else {
                *prev_next_ptr = ptr::null_mut();
            }

            (cur as *mut u8).offset(ibbsize())
        }
    }

    pub fn free(&self, ptr: *mut u8) {
        unsafe {
            let mut block = ptr.offset(-ibbsize()) as *mut FreeBlock;

            // try merge with previous
            let prev_block = (block as *mut u8).offset(-((*block).prev_size as isize) - ibbsize()) as *mut FreeBlock;
            let next_block = (block as *mut u8).offset(ibbsize() + (*block).size as isize) as *mut FreeBlock;

            if (*block).prev_size != 0 && (*prev_block).prev_size & 0x1 != 0 {
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
                (*next_block).prev_size & 0x1 != 0 { // it's indeed free
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
    }

    unsafe fn find_free_block(&self, size: u16) -> (*mut FreeBlock, *mut FreeBlock) {
        let mut prev = ptr::null_mut();
        let mut cur = *self.free_list_start();
        while !cur.is_null() && (*cur).size < size {
            prev = cur;
            cur = (*cur).next;
        }

        (prev, cur)
    }

    unsafe fn install_free_block(&self, block: *mut FreeBlock) {
        // TODO: maybe sort them by memory address when the size is same.
        // That will allow one neat optimization in the future
        let (prev, next) = self.find_free_block((*block).size);

        let prev_next = self.get_next_ptr(prev);

        *prev_next = block;
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
                    prev_size: 0 as u16,
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
        with_memory(32, |_, a| {
            let ret = a.alloc(32 - psize() - bbsize() + 1);

            assert_eq!(ptr::null_mut(), ret);
        });
    }

    #[test]
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
}
