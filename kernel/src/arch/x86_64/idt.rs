use spin::Once;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    arch::{gdt, x86_64::apic},
    drivers, hlt_loop, println,
};

pub static IDT: Once<InterruptDescriptorTable> = Once::new();

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 0x20,
    Keyboard = 0x21,
    Error = 0x70,
    Spurious = 0xf0,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        self as usize
    }
}

pub fn init() {
    let mut idt = InterruptDescriptorTable::new();

    // cpu interrupts
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault.set_handler_fn(gpf_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // external interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_int_handler);
    idt[InterruptIndex::Spurious.as_u8()].set_handler_fn(spurious_int_handler);
    idt[InterruptIndex::Error.as_u8()].set_handler_fn(error_int_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_int_handler);

    IDT.call_once(|| idt);

    unsafe {
        IDT.get_unchecked().load();
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{stack_frame:#?}")
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _err_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{stack_frame:#?}");
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, _err_code: u64) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{stack_frame:#?}");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", err_code);
    println!("{:#?}", stack_frame);

    hlt_loop();
}

extern "x86-interrupt" fn timer_int_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        apic::lapic_end_of_interrupt();
    }
}

extern "x86-interrupt" fn spurious_int_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        apic::lapic_end_of_interrupt();
    }
}

extern "x86-interrupt" fn error_int_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        apic::lapic_end_of_interrupt();
    }
}

extern "x86-interrupt" fn keyboard_int_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    drivers::keyboard::add_scancode(scancode);

    unsafe {
        apic::lapic_end_of_interrupt();
    }
}
