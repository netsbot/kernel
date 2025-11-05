#![no_std]
#![no_main]

mod io;

use crate::io::framebuffer;
use core::panic::PanicInfo;

bootloader_api::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    framebuffer::init_framebuffer_writer(boot_info.framebuffer.as_mut().unwrap());

    println!("hello, world!");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
