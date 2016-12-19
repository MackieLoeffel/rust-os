pub use self::range_allocator::RangeAllocator;
pub use self::paging::test_paging;
pub use self::paging::PAGE_TABLE;

mod range_allocator;
mod paging;

pub const FRAME_SIZE: usize = 4096;
pub type PhysicalAddress = usize;

// represents a physical frame
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame{ number: address / FRAME_SIZE }
    }

    fn next(&self) -> Frame {
        Frame {number: self.number + 1}
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * FRAME_SIZE
    }

    fn clone(&self) -> Frame {
        Frame {number: self.number}
    }
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn free(&mut self, frame: Frame);
}
