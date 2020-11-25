#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use core::panic::PanicInfo;
use rust_os::allocator;
use rust_os::memory;
use rust_os::qemu::{exit_qemu, QemuExitCode};
use rust_os::serial_println;

use x86_64::VirtAddr;

entry_point!(main);

use bootloader::{entry_point, BootInfo};
fn main(boot_info: &'static BootInfo) -> ! {
    let phys_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("[CRASH] Heap allocator failed");
    test_main();
    loop {}
}

use rust_os::allocator::HEAP_SIZE;

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn test_vec() {
    let mut v = Vec::new();
    let mut v2 = Vec::new();
    for i in 0..100 {
        v.push(i);
        v2.push(i);
    }
    assert_eq!(v, v2);
}

fn test_runner(tests: &[&dyn Fn()]) {
    // rust_os::test_runner(tests);
    exit_qemu(rust_os::qemu::QemuExitCode::Success)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
