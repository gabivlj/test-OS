use crate::gdt;
use crate::hlt_loop;
use crate::{print, println};
use lazy_static::lazy_static;

///
///  We will use for the IDT the already made struct in the
/// x86_64 crate that looks like this
///
/// #[repr(C)]
/// pub struct InterruptDescriptorTable {
///     pub divide_by_zero: Entry<HandlerFunc>,
///     pub debug: Entry<HandlerFunc>,
///     pub non_maskable_interrupt: Entry<HandlerFunc>,
///     pub breakpoint: Entry<HandlerFunc>,
///     pub overflow: Entry<HandlerFunc>,
///     pub bound_range_exceeded: Entry<HandlerFunc>,
///     pub invalid_opcode: Entry<HandlerFunc>,
///     pub device_not_available: Entry<HandlerFunc>,
///     pub double_fault: Entry<HandlerFuncWithErrCode>,
///     pub invalid_tss: Entry<HandlerFuncWithErrCode>,
///     pub segment_not_present: Entry<HandlerFuncWithErrCode>,
///     pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
///     pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
///     pub page_fault: Entry<PageFaultHandlerFunc>,
///     pub x87_floating_point: Entry<HandlerFunc>,
///     pub alignment_check: Entry<HandlerFuncWithErrCode>,
///     pub machine_check: Entry<HandlerFunc>,
///     pub simd_floating_point: Entry<HandlerFunc>,
///     pub virtualization: Entry<HandlerFunc>,
///     pub security_exception: Entry<HandlerFuncWithErrCode>,
/// }
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                // Set the interrupt stack index to swap to
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

///
/// Loads the DT, loading all the callbacks addresses to the IDT,
/// from the interrupts of the CPU to the external ones
///
pub fn init_dt() {
    IDT.load();
}

use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE ACCESS FAULT");
    println!("Accessed address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Exception Breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double Fault\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

use pic8259_simple::ChainedPics;
use spin;

/// Where we wanna store external interrupts
///
/// `32` is where the exception interrupts finish, 47 because there is 15 more
///
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 47;

///
/// PICS send us external interruptions
///
/// With the offsets we handle that the PICS send the interruptions
/// to the good place instead of the bad place :)
///
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Copy, Debug, Clone)]
#[repr(u8)]
///
/// InterruptIndex represents the position in which the interrupt
/// handler is located
///
pub enum InterruptIndex {
    ///
    /// timer interrupts by the PIC are on the line 0,
    /// so we set it to the offset
    ///
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

///
/// Timer interrupt handler
///
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    // print!(".");
    // Tell the PIC to notify that we handled the interrupt
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    // use crate::key::{Key, XT};
    // use core::convert::TryFrom;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);

    // let xt_key = XT(unsafe { port.read() });
    // let scancode: Result<Key, _> = Key::try_from(xt_key);
    // if let Ok(key) = scancode {
    //     print!("{}", key as u8 as char);
    // }

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let scancode: u8 = unsafe { port.read() };
    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
