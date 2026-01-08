use acpi::platform::interrupt::Apic;
use spin::Once;
use x2apic::{
    ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry},
    lapic::{LocalApic, LocalApicBuilder},
};
use x86_64::{PhysAddr, VirtAddr, instructions::port::Port};

use crate::{arch::idt::InterruptIndex, mem::phys_to_virt};

static LAPIC_BASE_ADDR: Once<u64> = Once::new();

/// # Safety
///
/// The caller must ensure that the provided Apic struct contains valid addresses for local apic and ioapic
pub unsafe fn init(apic: &Apic) {
    disable_8259_pics();

    let lapic = init_lapic(phys_to_virt(PhysAddr::new(apic.local_apic_address)));

    let first_ioapic_addr = apic.io_apics.first().expect("no ioapic found").address;

    init_ioapic(
        phys_to_virt(PhysAddr::new(first_ioapic_addr as u64)),
        &lapic,
    );
}

fn init_ioapic(ioapic_base_addr: VirtAddr, lapic: &LocalApic) {
    let mut ioapic = unsafe { IoApic::new(ioapic_base_addr.as_u64()) };

    // enables all entries
    // for i in 32..(255 - 32) {
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE);
        entry.set_dest(unsafe { lapic.id() } as u8);
        entry.set_vector(0x21);

        unsafe {
            ioapic.set_table_entry(0x1, entry);
            ioapic.enable_irq(0x1);
        }
    // }
}

fn init_lapic(lapic_base_addr: VirtAddr) -> LocalApic {
    LAPIC_BASE_ADDR.call_once(|| lapic_base_addr.as_u64());

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer.as_usize())
        .error_vector(InterruptIndex::Error.as_usize())
        .spurious_vector(InterruptIndex::Spurious.as_usize())
        .set_xapic_base(lapic_base_addr.as_u64())
        .build()
        .unwrap_or_else(|err| panic!("{}", err));

    unsafe { lapic.enable() }

    lapic
}

fn disable_8259_pics() {
    let mut master_pic = Port::new(0x21);
    let mut slave_pic = Port::new(0xa1);

    unsafe {
        master_pic.write(0xffu8);
        slave_pic.write(0xffu8);
    }
}

/// # Safety
///
/// This function must only be called after the LAPIC controller has been initialized
pub unsafe fn lapic_end_of_interrupt() {
    unsafe {
        let eoi_ptr = (LAPIC_BASE_ADDR.get_unchecked() + 0xb0) as *mut u32;
        core::ptr::write_volatile(eoi_ptr, 0);
    }
}
