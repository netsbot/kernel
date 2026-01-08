#![no_std]
#![no_main]
extern crate alloc;

use core::panic::PanicInfo;

use kernel::*;
use kernel::tasks::executor::ASYNC_EXECUTOR;

bootloader_api::entry_point!(kernel_start, config = &kernel::BOOTLOADER_CONFIG);
#[unsafe(no_mangle)]
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    drivers::init_stdout(&mut boot_info.framebuffer);
    mem::init(&boot_info.memory_regions);
    arch::init(boot_info.rsdp_addr);
    tasks::executor::init();
    drivers::init();

    println!("okay!");

    unsafe { ASYNC_EXECUTOR.get_unchecked() }.lock().run();

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");

    hlt_loop()
}
