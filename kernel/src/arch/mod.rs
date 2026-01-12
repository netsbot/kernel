#[cfg(target_arch = "x86_64")]
pub mod x86_64;

use ::acpi::platform::InterruptModel;
use limine::request::RsdpRequest;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub fn init() {
    gdt::init();
    idt::init();

    let rsdp_addr = RSDP_REQUEST
        .get_response()
        .expect("missing rsdp addr")
        .address();

    unsafe {
        acpi::init(rsdp_addr);

        if let InterruptModel::Apic(apic) = &acpi::ACPI_PLATFORM.get_unchecked().interrupt_model {
            apic::init(apic)
        } else {
            panic!("no xapic interrupt controller found")
        };
    }

    ::x86_64::instructions::interrupts::enable();
}
