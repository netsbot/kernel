#![no_std]
#![no_main]

mod io;

use crate::io::framebuffer::FrameBufferWriter;
use core::fmt::Write;
use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello, world!\n";

bootloader_api::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let fb = boot_info.framebuffer.as_mut().unwrap();

    let mut a = FrameBufferWriter::new(fb);
    for i in 0..100 {
        a.write_str("hello").unwrap();
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
