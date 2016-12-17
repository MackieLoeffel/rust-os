use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter};
use core::cmp::max;

pub struct RangeAllocator {
    next_free_frame: Frame,
    end: Frame,
}

impl RangeAllocator {
    #[allow(unused_variables)]
    pub fn new(areas: MemoryAreaIter,
               kernel_start: usize, kernel_end: usize,
               multiboot_start: usize, multiboot_end: usize) -> RangeAllocator {
        let biggest_area = areas.max_by_key(|area| area.length).unwrap();
        let heap_end = (biggest_area.base_addr + biggest_area.length) as usize;
        let heap_start = max(biggest_area.base_addr as usize, max(kernel_end, multiboot_end));
        assert!(heap_start < heap_end);

        RangeAllocator {
            next_free_frame: Frame::containing_address(heap_start - 1).next(),
            end: Frame::containing_address(heap_end)
        }
    }
}

impl FrameAllocator for RangeAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        if self.next_free_frame != self.end {
            let new = self.next_free_frame.clone();
            self.next_free_frame = self.next_free_frame.next();
            return Some(new);
        }
        return None;
    }

    #[allow(unused_variables)]
    fn free(&mut self, frame: Frame) {}
}
