use talc::{ErrOnOom, Span, Talc, Talck};
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

const HEAP_START: usize = 0x_4444_4444_0000;
const HEAP_INITIAL_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ErrOnOom> = Talc::new(ErrOnOom).lock();

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_INITIAL_SIZE as u64 - 1;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in heap_page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    let span = Span::new(
        HEAP_START as *mut u8,
        (HEAP_START + HEAP_INITIAL_SIZE) as *mut u8,
    );

    unsafe { ALLOCATOR.lock().claim(span) }
        .expect("there is too little memory to initialize the heap");

    Ok(())
}
