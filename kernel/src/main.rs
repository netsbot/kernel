#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::mem::MAPPER;
use kernel::{
    gdt, hlt_loop, interrupts,
    io::{framebuffer, serial_monitor},
    mem, println,
};
use x86_64::VirtAddr;
use x86_64::structures::paging::Page;

bootloader_api::entry_point!(kernel_start, config = &kernel::BOOTLOADER_CONFIG);
#[unsafe(no_mangle)]
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    framebuffer::init_framebuffer_writer(boot_info.framebuffer.as_mut().unwrap());
    serial_monitor::init_serial_monitor_writer(kernel::SERIAL_MONITOR_PORT);
    mem::init();
    gdt::init_gdt();
    interrupts::init_idt();

    let mut frame_allocator = mem::EmptyFrameAllocator;

    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    unsafe {
        let mapper = MAPPER.get_unchecked();

        mem::create_example_mapping(page, &mut mapper.lock(), &mut frame_allocator);
        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);

        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        let a = page_ptr.offset(400).read();
        println!("{a:#x}")
    }

    // let handler = kernel::io::acpi::AcpiHandler::new(VirtAddr::new(PHYSICAL_MEMORY_OFFSET));
    // let rdsp = boot_info.rsdp_addr.take().unwrap();
    //
    // unsafe {
    //     let table = acpi::AcpiTables::from_rsdp(handler, rdsp as usize).unwrap();
    // }

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
