pub mod frame_allocator;
pub mod heap;

use core::ops::DerefMut;

use bootloader_api::info::MemoryRegions;
pub use frame_allocator::FRAME_ALLOCATOR;
use spin::{Mutex, Once};
use x86_64::{
    registers::control::Cr3, structures::paging::{OffsetPageTable, PageTable, Translate},
    PhysAddr,
    VirtAddr,
};

pub static MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();

pub fn init(memory_map: &'static MemoryRegions) {
    unsafe {
        let level_4_table = active_level_4_table();
        MAPPER.call_once(|| {
            Mutex::new(OffsetPageTable::new(
                level_4_table,
                VirtAddr::new(PHYSICAL_MEMORY_OFFSET),
            ))
        });

        frame_allocator::init_frame_allocator(memory_map);

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
    VirtAddr::new(addr.as_u64() + PHYSICAL_MEMORY_OFFSET)
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

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF_8000_0000_0000;