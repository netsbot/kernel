use bootloader_api::info::{MemoryRegionKind::Usable, MemoryRegions};
use spin::{Mutex, Once};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
};

pub static FRAME_ALLOCATOR: Once<Mutex<KernelFrameAllocator>> = Once::new();

/// # Safety
///
/// The caller must ensure that all the Usable regions in memory_map is unused
pub unsafe fn init_frame_allocator(memory_map: &'static MemoryRegions) {
    FRAME_ALLOCATOR.call_once(|| unsafe { Mutex::new(KernelFrameAllocator::new(memory_map)) });
}

pub struct KernelFrameAllocator {
    bitmap: &'static mut [u64],
    max_frames: u64,
    // optimization so that we do not search over already allocated frames
    next_free_frame: u64,
}

impl KernelFrameAllocator {
    /// # Safety
    ///
    /// The caller must ensure that all the Usable regions in memory_map is unused
    pub unsafe fn new(memory_map: &'static MemoryRegions) -> Self {
        let total_frames = memory_map.iter().last().unwrap().end / 4096;
        let bitmap_size = total_frames.div_ceil(8);

        let bitmap_memory_region = memory_map
            .iter()
            .find(|r| r.kind == Usable && (r.end - r.start) >= bitmap_size)
            .expect("could not find a memory region large enough for the bitmap");

        let bitmap_start_addr = bitmap_memory_region.start;

        let bitmap_slice = unsafe {
            core::slice::from_raw_parts_mut(
                super::phys_to_virt(PhysAddr::new(bitmap_start_addr)).as_mut_ptr(),
                bitmap_size as usize / 8,
            )
        };

        // start from safe state (everything is used)
        bitmap_slice.fill(0xFF);

        let mut allocator = Self {
            bitmap: bitmap_slice,
            max_frames: total_frames,
            next_free_frame: 0,
        };

        memory_map
            .iter()
            .filter(|r| r.kind == Usable)
            .for_each(|r| allocator.mark_range_as_free(r.start, r.end));

        // mark the bitmap memory as used
        allocator.mark_range_as_used(bitmap_start_addr, bitmap_start_addr + bitmap_size);
        allocator.next_free_frame = (bitmap_start_addr + bitmap_size) / 4096;

        allocator
    }

    fn mark_range_as_free(&mut self, start: u64, end: u64) {
        for frame_addr in (start..end).step_by(4096) {
            let frame_idx = (frame_addr / 4096) as usize;
            self.bitmap[frame_idx / 64] &= !(1 << (frame_idx % 64));
        }
    }

    fn mark_range_as_used(&mut self, start: u64, end: u64) {
        for frame_addr in (start..end).step_by(4096) {
            let frame_idx = (frame_addr / 4096) as usize;
            self.bitmap[frame_idx / 64] |= 1 << (frame_idx % 64);
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for frame_idx in self.next_free_frame..self.max_frames {
            let byte_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;

            if (self.bitmap[byte_idx as usize] & (1 << bit_idx)) == 0 {
                self.bitmap[byte_idx as usize] |= 1 << bit_idx;
                self.next_free_frame += 1;

                let addr = frame_idx * 4096;
                return Some(PhysFrame::containing_address(PhysAddr::new(addr)));
            }
        }

        None // no free frames
    }
}

impl FrameDeallocator<Size4KiB> for KernelFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame_idx = frame.start_address().as_u64() / 4096;
        let byte_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;

        self.bitmap[byte_idx as usize] &= !(1 << bit_idx);

        if frame_idx < self.next_free_frame {
            self.next_free_frame = frame_idx
        }
    }
}
