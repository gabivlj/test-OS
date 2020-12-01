use crate::println;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::PhysAddr;
use x86_64::{structures::paging::PageTable, VirtAddr};

/// Initialize a new offset pages table so we can fill it later
/// with map
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

///
/// Returns a mutable reference to the active level 4 table
///
/// The physical address needs to be the offset where the tables start
///
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

///
/// Translates the given addr to the mapped phys address
///
/// `None` if the addr is not mapped.
///
/// It is unsafe because the memory offset must be right.
///
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

///
/// Example on how translating a physical memory offset type table map
/// could work
///
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;
    for &index in &table_indexes {
        // get the virtual address where the page frame is located
        // because we map the entire physical addresses there is only an offset
        let virt = physical_memory_offset + frame.start_address().as_u64();
        // cast
        let table_ptr: *const PageTable = virt.as_ptr();
        // cast again
        let table = unsafe { &*table_ptr };
        // get the entry so we go to the next frame
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge frame!!"),
        }
    }
    // we are finally on the frame of physical memory, now we add the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB};

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

const SIZE_USABLE_FRAMES: usize = 32007;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
    available_frames: [Option<PhysFrame>; SIZE_USABLE_FRAMES],
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map and recollects all the available frames.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        let mut i = 0;
        let mut usable_frames: [Option<PhysFrame>; SIZE_USABLE_FRAMES] = [None; SIZE_USABLE_FRAMES];
        let mut boot_info_frame_allocator = Self {
            memory_map,
            next: 0,
            available_frames: usable_frames,
        };
        for frame in boot_info_frame_allocator.usable_frames() {
            if i >= SIZE_USABLE_FRAMES {
                println!("[WARNING] We are not covering all the available memory, that's ok though. If we want more memory set the constant to support more physical pages \nNumber of frames: {}. Available ones: {:?}", i, boot_info_frame_allocator.usable_frames().count() );
                break;
            }
            boot_info_frame_allocator.available_frames[i] = Some(frame);
            i += 1;
        }
        boot_info_frame_allocator
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    ///
    /// Checks in the available_frames table the next physical memory frame to map
    ///
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.available_frames[self.next];
        self.next += 1;
        frame
    }
}
