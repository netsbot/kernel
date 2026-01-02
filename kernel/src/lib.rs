#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;

pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod mem;

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF_8000_0000_0000;
pub const SERIAL_MONITOR_PORT: u16 = 0x3F8;

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYSICAL_MEMORY_OFFSET));
    config
};

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
