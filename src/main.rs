#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use core::panic::PanicInfo;
use rust_os::allocator;
use rust_os::memory;
use rust_os::println;
use rust_os::test_panic_handler;

fn recursive_virt_addr() {
    let addr: usize = 0x32334584;
    let r = 0o777; // recursive index
    let sign = 0o1777777 << 48; // sign
                                // retrieve the page table indices of the add that we want to translate
    let l4_idx = (addr >> 39) & r; // level 4 index;
    let l3_idx = (addr >> 30) & r; // level 3 index;
    let l2_idx = (addr >> 21) & r; // level 2 index;
    let l1_idx = (addr >> 12) & r; // level 1 index;

    // calculate addresses
    let level_4_table_addr = sign | (r << 39) | (r << 30) | (r << 21) | (r << 12);
    let level_3_table_addr = sign | (r << 39) | (r << 30) | (r << 21) | (l4_idx << 12);
    let level_2_table_addr = sign | (r << 39) | (r << 30) | (l4_idx << 21) | (l3_idx << 12);
    let level_1_table_addr = sign | (r << 39) | (l4_idx << 30) | (l3_idx << 21) | (l2_idx << 12);
}

use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::{
        structures::paging::{MapperAllSizes, Page},
        VirtAddr,
    };
    println!("Welcome to this OS{} Initializing some stuff. . .", "!");

    #[cfg(test)]
    test_main();
    rust_os::init_os();
    // Physical Offset (Where the tables are places)
    // from the bootloader, tell the mapper to initialize working with that
    // let (mut mapper, mut frame_allocator) = rust_os::init_with_frame_alloc(boot_info);
    let phys_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // Initialize table struct
    let mut mapper = unsafe { memory::init(phys_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("[CRASH] Heap allocator failed");
    // Showing that the pages from the bootloader work
    // println!(
    //     "0xb8000 -> {:?}",
    //     mapper.translate_addr(VirtAddr::new(0xb8000))
    // );
    // Get the virtual page that contains this virtual addr (Basically get the frame to map)
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    // map VGA 0xb8000 to the page where 0xdeadbeef
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // // access that virtual address and write to it
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe {
    //     page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    // };
    // Show that translation works as expected
    // println!(
    //     "{:?} -> {:?}",
    //     VirtAddr::new(page.start_address().as_u64()),
    //     unsafe { mapper.translate_addr(VirtAddr::new(page.start_address().as_u64())) }
    // );
    println!("Welcome :) Everything is fine");
    rust_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}
