#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod interrupts;
pub mod io;
pub mod gdt;
