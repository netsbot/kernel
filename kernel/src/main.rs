#![no_std]
#![no_main]
extern crate alloc;

use core::{ops::DerefMut, panic::PanicInfo};

use kernel::{arch::gdt::GDT, mem::virt_to_phys, tasks::executor::ASYNC_EXECUTOR, *};
use x86_64::{
    VirtAddr,
    registers::rflags::RFlags,
    structures::{
        gdt::SegmentSelector,
        idt::InterruptStackFrameValue,
        paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
    },
};

bootloader_api::entry_point!(kernel_start, config = &BOOTLOADER_CONFIG);
#[unsafe(no_mangle)]
fn kernel_start(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    drivers::init_stdout(&mut boot_info.framebuffer);
    mem::init(&boot_info.memory_regions);
    arch::init(boot_info.rsdp_addr);
    tasks::executor::init();
    drivers::init();

    println!("okay!");

    static mut USER_STACK: [u8; 4096] = [0; 4096];
    #[allow(static_mut_refs)]
    let stack_start = VirtAddr::from_ptr(unsafe { USER_STACK.as_ptr() });

    let stack: Page = Page::containing_address(VirtAddr::new(0x0000_7000_0000_0000));
    let func: Page = Page::containing_address(VirtAddr::new(0x0000_6000_0000_0000));

    unsafe {
        let phys_addr = virt_to_phys(stack_start).unwrap();
        let frame = PhysFrame::containing_address(phys_addr);

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        mem::MAPPER
            .get_unchecked()
            .lock()
            .map_to(
                stack,
                frame,
                flags,
                mem::FRAME_ALLOCATOR.get_unchecked().lock().deref_mut(),
            )
            .expect("fuck")
            .flush();

        let phys_addr = virt_to_phys(VirtAddr::new(mock_user_main as u64)).unwrap();
        let frame = PhysFrame::containing_address(phys_addr);

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        mem::MAPPER
            .get_unchecked()
            .lock()
            .map_to(
                func,
                frame,
                flags,
                mem::FRAME_ALLOCATOR.get_unchecked().lock().deref_mut(),
            )
            .unwrap()
            .flush();
    }

    let func_offset = (mock_user_main as u64) & 0xFFF;
    let user_rip = 0x0000_6000_0000_0000 + func_offset;

    let frame = InterruptStackFrameValue::new(
        VirtAddr::new(user_rip),
        SegmentSelector(27),
        RFlags::INTERRUPT_FLAG,
        VirtAddr::new(0x0000_7000_0000_0000 + 4096),
        SegmentSelector(35),
    );

    println!("mapped");

    unsafe {
        core::arch::asm!(
        "mov rsp, {0}",
        "iretq",
        in(reg) &frame as *const _ as u64,
        options(noreturn)
        );
    }

    unsafe { ASYNC_EXECUTOR.get_unchecked() }.lock().run();

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");

    hlt_loop()
}

pub extern "C" fn mock_user_main() -> ! {
    // let cs: u16;
    // unsafe {
    //     core::arch::asm!("mov {0:x}, cs", out(reg) cs);
    // }
    //
    // // If we are in Ring 3, the bottom 2 bits of CS will be 11 (3)
    // if (cs & 0x3) == 3 {
    //     // SUCCESS: We are in Ring 3!
    //     // Since we have no print, we can signal success via a port or a magic value in a register
    unsafe {
        core::arch::asm!("mov rax, 0x1337", "hlt");
    }
    // } else {
    //     // FAILURE: Still in Ring 0
    //     loop {}
    // }
    //
    loop {}
}
