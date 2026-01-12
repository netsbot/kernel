use core::arch::asm;

use spin::Once;
use x86_64::{
    VirtAddr,
    instructions::{
        segmentation::{CS, Segment},
        tables::load_tss,
    },
    registers::segmentation::{DS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub static TSS: Once<TaskStateSegment> = Once::new();
pub static GDT: Once<GlobalDescriptorTable> = Once::new();

fn init_tss() {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + STACK_SIZE as u64 // stack end
    };

    let rsp: u64;
    unsafe {
        asm!(
        "mov {}, rsp",
        out(reg) rsp
        );
    }

    tss.privilege_stack_table[0] = VirtAddr::new(rsp);

    TSS.call_once(|| tss);
}

pub fn init() {
    init_tss();

    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());

    let user_code_selector = gdt.append(Descriptor::user_code_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());

    let tss_selector = gdt.append(Descriptor::tss_segment(
        TSS.get().expect("tss not initialized"),
    ));

    GDT.call_once(|| gdt);
    GDT.get().unwrap().load();

    unsafe {
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        SS::set_reg(data_selector);
        load_tss(tss_selector);
    }
}
