const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

struct Node {
    next: Option<&'static mut Node>,
}

pub struct FixedBlockAllocator {
    list_heads: [Option<&'static mut Node>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedBlockAllocator {
    pub const fn new() -> Self {
        Self {
            list_heads: [None; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    ///
    /// Initializes the allocator
    ///
    /// Unsafe because it must guarantee that the heap ranges are correct
    ///
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }
}

use alloc::alloc::Layout;
use core::ptr;

impl FixedBlockAllocator {
    fn fallback_allocator(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn fallback_dealloc(&mut self, layout: Layout, pointer: *mut u8) {
        self.fallback_allocator.deallocate(
            ptr::NonNull::new(pointer).expect("it should be non-null"),
            layout,
        );
    }
}

///
/// Returns the desired block size idx
///
fn list_index(layout: &Layout) -> Option<usize> {
    // We want it aligned
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&x| x >= required_block_size)
}

use super::{align_up, Locked};
use alloc::alloc::GlobalAlloc;

unsafe impl GlobalAlloc for Locked<FixedBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator: spin::MutexGuard<FixedBlockAllocator> = self.lock();
        match list_index(&layout) {
            Some(index) => match allocator.list_heads[index].take() {
                Some(node) => {
                    // go next
                    allocator.list_heads[index] = node.next.take();
                    // return current
                    node as *mut Node as *mut u8
                }
                None => {
                    // No more blocks, allocate one on fallback
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;
                    // Align still to a block alignment because it will be added
                    // to a head later when it's deallocated.
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    allocator.fallback_allocator(layout)
                }
            },
            None => allocator.fallback_allocator(layout),
        }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        use core::mem;
        let mut allocator: spin::MutexGuard<FixedBlockAllocator> = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let old_head = allocator.list_heads[index].take();
                let mut node = Node { next: old_head };
                assert!(mem::size_of::<Node>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<Node>() <= BLOCK_SIZES[index]);
                let ptr = pointer as *mut Node;
                ptr.write(node);
                allocator.list_heads[index] = Some(&mut *ptr);
            }
            None => {
                allocator.fallback_dealloc(layout, pointer);
            }
        }
    }
}
