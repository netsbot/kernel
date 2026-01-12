#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
extern crate alloc;

pub mod arch;
pub mod common;
pub mod drivers;
pub mod mem;
pub mod tasks;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
