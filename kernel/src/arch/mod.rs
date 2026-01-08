#[cfg(target_arch = "x86_64")]
pub mod x86_64;

use ::acpi::platform::InterruptModel;
use bootloader_api::BootInfo;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;


pub fn init(rsdp_addr: bootloader_api::info::Optional<u64>) {
    gdt::init();
    idt::init();

    unsafe {
        acpi::init(
            rsdp_addr
                .into_option()
                .expect("missing rsdp addr"),
        );

        if let InterruptModel::Apic(apic) =
            &acpi::ACPI_PLATFORM.get_unchecked().interrupt_model
        {
            apic::init(apic)
        } else {
            panic!("no xapic interrupt controller found")
        };
    }

    ::x86_64::instructions::interrupts::enable();
}
