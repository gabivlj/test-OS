#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;
use rust_os::test_panic_handler;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Welcome to this OS{}", "!");
    #[cfg(test)]
    test_main();

    rust_os::init();

    fn stackoverflow() {
        stackoverflow();
    }
    stackoverflow();

    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // };

    x86_64::instructions::interrupts::int3();

    println!("NOT crashed!");

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}
