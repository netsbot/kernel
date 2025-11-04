#![no_std]
#![no_main]

mod io;

use crate::io::framebuffer::FrameBufferWriter;
use bootloader_api::info::Optional;
use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello, world!\n";

bootloader_api::entry_point!(kernel_start);
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let fb = match &mut boot_info.framebuffer {
        Optional::Some(fb) => fb,
        Optional::None => panic!(),
    };

    let w = fb.info().width;
    let h = fb.info().height;

    let mut a = FrameBufferWriter::new(fb);
    for i in 0..h {
        for j in 0..w {
            a.write_pixel(j, i);
        }
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
