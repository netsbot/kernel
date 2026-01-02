use crate::PHYSICAL_MEMORY_OFFSET;
use spin::{Mutex, Once};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub static MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();

pub fn init() {
    unsafe {
        let level_4_table = active_level_4_table();
        MAPPER.call_once(|| {
            Mutex::new(OffsetPageTable::new(
                level_4_table,
                VirtAddr::new(PHYSICAL_MEMORY_OFFSET),
            ))
        });
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// # Safety
///
/// This function must only be called once to avoid aliasing &mut references.
unsafe fn active_level_4_table() -> &'static mut PageTable {
    let (level_4_page_frame, _) = Cr3::read();

    let virt = phys_to_vert(level_4_page_frame.start_address());
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// This assumes that the bootloader supports paging, otherwise, the returned virtual address
/// when used will cause a page fault
pub fn phys_to_vert(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + PHYSICAL_MEMORY_OFFSET)
}
