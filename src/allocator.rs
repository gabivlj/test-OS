#![feature(alloc_error_handler)]
#[global_allocator]
static ALLOCATOR: Locked<FixedBlockAllocator> = Locked::new(FixedBlockAllocator::new());
pub mod bump;
pub mod fixed_block;
pub mod linked_list;

use crate::println;
use alloc::alloc::{GlobalAlloc, Layout};
use bump::BumpAllocator;
use core::ptr::null_mut;
use fixed_block::FixedBlockAllocator;
use linked_list::LinkedListAllocator;
use linked_list_allocator::LockedHeap;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = VirtAddr::new(HEAP_START as u64 + HEAP_SIZE as u64 - 1);
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // Create/Map to table pages
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

///
/// A dummy allocator that only returns null pointers
///
pub struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc what? baby, it's all null pointers, it always has been...");
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

use spin;

/// Safe abstraction for mutexes and allows impl traits
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

///
/// Aligns address with the desired alignment (rounds up)
///
pub fn align_up(addr: usize, align: usize) -> usize {
    // println!("{} aligned", align);
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}
