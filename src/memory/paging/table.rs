use core::marker::{PhantomData};
use core::ops::{Index, IndexMut, Deref, DerefMut};
use spin::Mutex;
use x86::shared::tlb;
use x86::shared::control_regs;
use super::entry::*;
use super::{Page, InactivePageTable};
use super::mapper::{Mapper};
use super::temporary_page::{TemporaryPage};
use memory::{Frame, FrameAllocator, PhysicalAddress};

pub const ENTRY_COUNT: usize = 512;

pub static PAGE_TABLE: Mutex<PageTable> = Mutex::new(PageTable {
    mapper: unsafe {Mapper::new()}
});

pub struct PageTable {
    mapper: Mapper,
}

impl PageTable {
    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut TemporaryPage,
                   f:F ) where F: FnOnce(&mut Mapper) {
        {
            let backup = Frame::containing_address(
                unsafe {control_regs::cr3()}
            );

            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // override recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            unsafe {tlb::flush_all();}

            f(self);

            p4_table[511].set(backup, PRESENT | WRITABLE);
            unsafe {tlb::flush_all();}

        }

        temporary_page.unmap(self);
    }
}

impl Deref for PageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper { &self.mapper }
}

impl DerefMut for PageTable {
    fn deref_mut(&mut self) -> &mut Mapper { &mut self.mapper }
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
