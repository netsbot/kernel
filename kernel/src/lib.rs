#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::{config::Mapping, BootloaderConfig};
use mem::PHYSICAL_MEMORY_OFFSET;

pub mod arch;
pub mod drivers;
pub mod mem;
pub mod tasks;

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
