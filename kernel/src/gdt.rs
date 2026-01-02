use spin::Once;
use x86_64::VirtAddr;
use x86_64::instructions::{
    segmentation::{CS, Segment},
    tables::load_tss,
};
use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable},
    tss::TaskStateSegment,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub static TSS: Once<TaskStateSegment> = Once::new();
pub static GDT: Once<GlobalDescriptorTable> = Once::new();

fn init_tss() {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = unsafe {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + STACK_SIZE as u64 // stack end
    };

    TSS.call_once(|| tss);
}

pub fn init_gdt() {
    init_tss();

    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(
        TSS.get().expect("tss not initialized"),
    ));

    GDT.call_once(|| gdt);
    GDT.get().unwrap().load();

    unsafe {
        CS::set_reg(code_selector);
        load_tss(tss_selector);
    }
}
