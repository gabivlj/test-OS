#![feature(wake_trait)]
#![feature(box_into_pin)]
#![feature(raw)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#[cfg(not(test))]
use bootloader::BootInfo;
use x86_64::{
    structures::paging::{FrameAllocator, MapperAllSizes, OffsetPageTable, Page, Size4KiB},
    VirtAddr,
};

// All of the components of the so
pub mod allocator;
pub mod gdt;
mod interrupts;
pub mod key;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod vga_buffer;

use core::panic::PanicInfo;

extern crate alloc;

pub fn init_os() {
    gdt::init();
    interrupts::init_dt();
    // Initialize PICS so we know where the external interrupts are going
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

///
/// Initializes the core of the OS plus
/// the frame allocator, the page tables and heap allocator
/// using the physical address offset and the memory_map (available physical addresses)
/// of the OS
///
/// ! This crashes... and I dont know why!
///
#[inline]
pub fn init_with_frame_alloc(
    boot_info: &'static BootInfo,
) -> (OffsetPageTable<'static>, impl FrameAllocator<Size4KiB>) {
    init_os();
    let phys = VirtAddr::new(boot_info.physical_memory_offset);
    // Initialize table struct
    let mut map = unsafe { memory::init(phys) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut map, &mut frame_allocator).expect("[CRASH] Heap allocator failed");
    (map, frame_allocator)
}

///
/// Sleeps CPU until the next interruption comes
///
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

///
/// Testing
///
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
pub fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init_os();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
