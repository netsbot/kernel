#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::{gdt, interrupts, io::framebuffer, println};

bootloader_api::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    framebuffer::init_framebuffer_writer(boot_info.framebuffer.as_mut().unwrap());
    gdt::init_gdt();
    interrupts::init_idt();
    
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    framebuffer::WRITER.get().unwrap().lock();
    println!("{info}");

    loop {}
}
