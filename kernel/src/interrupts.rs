use crate::{gdt, println};
use once_cell_no_std::OnceCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub static IDT: OnceCell<InterruptDescriptorTable> = OnceCell::new();

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    IDT.set(idt)
        .expect("idt already initialized")
        .expect("concurrent access");

    IDT.get().unwrap().load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{stack_frame:#?}")
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _err_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{stack_frame:#?}");

    loop {}
}
