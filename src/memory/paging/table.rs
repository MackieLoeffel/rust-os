use core::marker::{PhantomData};
use core::ops::{Index, IndexMut};
use core::ptr::{Unique};
use spin::Mutex;
use memory::paging::entry::*;
use memory::paging::{Page, VirtualAddress, PAGE_SIZE};
use memory::{Frame, FrameAllocator, PhysicalAddress};

const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;
pub const ENTRY_COUNT: usize = 512;

pub static PAGE_TABLE: Mutex<PageTable> = Mutex::new(PageTable {
    p4: unsafe {Unique::new(P4)}});

pub struct PageTable {
    p4: Unique<Table<Level4>>
}

impl PageTable {
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.start_address() + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() *
                                ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index()
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].frame())
            .or_else(huge_page)
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags,
                     allocator: &mut A) where A : FrameAllocator {
        assert!(!flags.contains(HUGE_PAGE), "HUGE pages are not supported for mapping");
        let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);
        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags,
                  allocator: &mut A) where A : FrameAllocator {
        let frame = allocator.alloc().expect("Out of Memory!");
        self.map_to(page, frame, flags, allocator);
    }

    pub fn identity_map<A>(&mut self, page: Page, flags: EntryFlags,
                           allocator: &mut A) where A : FrameAllocator {
        let frame = Frame::containing_address(page.start_address());
        self.map_to(page, frame, flags, allocator);
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].frame().unwrap();
        p1[page.p1_index()].set_unused();
        // TODO free p(1,2,3) table if empty

        // flush tlb
        unsafe {
            asm!("invlpg ($0)" :: "r" (page.start_address()) : "memory");
        }
        allocator.free(frame);
    }

    fn p4(&self) -> &Table<Level4> { unsafe { self.p4.get() }}
    fn p4_mut(&mut self) -> &mut Table<Level4> { unsafe { self.p4.get_mut() }}
}

// trick, that the compiler forbids gettings the next table from a P1 table
pub trait TableLevel {}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel { type NextLevel: TableLevel; }

impl HierarchicalLevel for Level4 { type NextLevel = Level3; }
impl HierarchicalLevel for Level3 { type NextLevel = Level2; }
impl HierarchicalLevel for Level2 { type NextLevel = Level1; }

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>
}

impl<L> Table<L> where L: TableLevel {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L> Table<L> where L: HierarchicalLevel {
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A)
                                -> &mut Table<L::NextLevel>
        where A : FrameAllocator {
        if self.next_table_mut(index).is_none() {
            assert!(!self[index].flags().contains(HUGE_PAGE), "Huge pages not supported!");
            let table_frame = allocator.alloc().expect("Out of Memory!");
            self[index].set(table_frame, PRESENT | WRITABLE);
            self.next_table_mut(index).unwrap().zero()
        }

        self.next_table_mut(index).unwrap()
    }

    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}
