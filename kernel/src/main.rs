#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::{
    gdt, hlt_loop, interrupts,
    io::{framebuffer, serial_monitor},
    mem, println,
};

bootloader_api::entry_point!(kernel_start, config = &kernel::BOOTLOADER_CONFIG);
#[unsafe(no_mangle)]
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    framebuffer::init_framebuffer_writer(boot_info.framebuffer.as_mut().unwrap());
    serial_monitor::init_serial_monitor_writer(kernel::SERIAL_MONITOR_PORT);
    mem::init();
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe {
        mem::frame_allocator::init_frame_allocator(&boot_info.memory_regions);
    }

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // if our panic handler fails, there is no saving
    unsafe {
        framebuffer::WRITER.get_unchecked().lock();
        serial_monitor::WRITER.get_unchecked().lock();
    }
    println!("{info}");

    hlt_loop()
}
