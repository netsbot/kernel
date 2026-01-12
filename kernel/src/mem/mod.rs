pub mod frame_allocator;
pub mod heap;

use core::ops::DerefMut;

pub use frame_allocator::FRAME_ALLOCATOR;
use limine::request::HhdmRequest;
use spin::{Mutex, Once};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable, Translate},
};

use crate::println;

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub static MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();

pub fn init() {
    unsafe {
        let level_4_table = active_level_4_table();
        MAPPER.call_once(|| {
            Mutex::new(OffsetPageTable::new(
                level_4_table,
                VirtAddr::new(
                    HHDM_REQUEST
                        .get_response()
                        .expect("hhdm not enabled")
                        .offset(),
                ),
            ))
        });

        frame_allocator::init_frame_allocator();

        heap::init_heap(
            MAPPER.get_unchecked().lock().deref_mut(),
            FRAME_ALLOCATOR.get_unchecked().lock().deref_mut(),
        )
        .expect("heap initialization failed");
    }
}

/// # Safety
///
/// This function must only be called once to avoid aliasing &mut references.
unsafe fn active_level_4_table() -> &'static mut PageTable {
    let (level_4_page_frame, _) = Cr3::read();

    let virt = phys_to_virt(level_4_page_frame.start_address());
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(
        addr.as_u64()
            + HHDM_REQUEST
                .get_response()
                .expect("hhdm not enabled")
                .offset(),
    )
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        MAPPER
            .get()
            .expect("mapper not initialized")
            .lock()
            .translate_addr(addr)
    })
}

#[macro_export]
macro_rules! map_page {
    ($phys:expr, $virt:expr, $size:ty, $flags:expr) => {
        // macros expect everything to be imported each time they're used in a new file, so best to hardcode paths
        let phys_frame = x86_64::structures::paging::PhysFrame::containing_address($phys);
        let page = x86_64::structures::paging::Page::<$size>::containing_address($virt);

        x86_64::instructions::interrupts::without_interrupts(|| {
            // suppress warnings if this macro is called from an unsafe fn
            #[allow(unused_unsafe)]
            let res = unsafe {
                // in case this macro is called from a file that doesn't import this
                use x86_64::structures::paging::Mapper as MacroMapper;

                $crate::mem::MAPPER.get().expect("mapper not initialized").lock().map_to(
                    page,
                    phys_frame,
                    $flags,
                    &mut *$crate::mem::FRAME_ALLOCATOR.get().expect("frame allocator not initialized").lock(),
                )
            };

            let flush = match res{
               Ok(flush) => Some(flush),
                Err(e) => match e {
                    x86_64::structures::paging::mapper::MapToError::FrameAllocationFailed => panic!("Out of memory"),
                    x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(_) => {
                        // Skip mapping as page already exists
                        None
                    }
                    x86_64::structures::paging::mapper::MapToError::ParentEntryHugePage => {
                        // Skip mapping as page already exists
                        None
                    }
                },
            };

            if let Some(flush) = flush {
                flush.flush();
            }
        });
    };
}
