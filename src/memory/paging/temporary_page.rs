use super::{Page, PageTable, VirtualAddress};
use memory::{Frame, FrameAllocator};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn map(&mut self, frame: Frame, active_table: &mut PageTable) -> VirtualAddress {
        use super::entry::WRITABLE;

        assert!(active_table.translate_page(self.page).is_none(),
                "already mapped");
        active_table.map_to(self.page, frame, WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut PageTable) {
        active_table.unmap(self.page, &mut self.allocator);
    }
}

// one for P3, P2 and P1
struct TinyAllocator([Option<Frame>; 3]);

impl FrameAllocator for TinyAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn free(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames");
    }
}
