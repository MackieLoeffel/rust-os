pub use self::table::PAGE_TABLE;

mod entry;
mod table;
mod temporary_page;
mod mapper;

use multiboot2::BootInformation;
use self::entry::EntryFlags;
use self::entry::{PRESENT, WRITABLE};
use memory::{FrameAllocator, FRAME_SIZE, Frame};
use self::table::{PageTable};
use self::temporary_page::{TemporaryPage};

// must be the same
pub const PAGE_SIZE: usize = FRAME_SIZE;
pub type VirtualAddress = usize;

// represents a virtual page
#[derive(Debug, Copy, Clone)]
pub struct Page {
   number: usize,
}

impl Page {
    fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
                address >= 0xffff_8000_0000_0000,
                "invalid address: {:#x}", address);
        Page {number: address / PAGE_SIZE}
    }

    fn p4_index(&self) -> usize { (self.number >> 27) & 0o777 }
    fn p3_index(&self) -> usize { (self.number >> 18) & 0o777 }
    fn p2_index(&self) -> usize { (self.number >> 9) & 0o777 }
    fn p1_index(&self) -> usize { (self.number >> 0) & 0o777 }

    fn start_address(&self) -> usize { self.number * PAGE_SIZE }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame,
               active_table: &mut PageTable,
               temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            // setup recursive mapping
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable {p4_frame: frame}
    }
}

pub fn remap_the_kernel<A>(active_table: &mut PageTable,
                           allocator: &mut A,
                           boot_info: &BootInformation)
    where A: FrameAllocator
{
    let tmp_page_page = Page::containing_address(0xcafebabe);
    assert!(active_table.translate_page(tmp_page_page.clone()).is_none());
    let mut tmp_page = TemporaryPage::new(tmp_page_page, allocator);

    // we leak one frame here, but we don't care
    let tmp_frame = allocator.alloc().expect("out of memory");
    let mut new_page_table = InactivePageTable::new(tmp_frame, active_table, &mut tmp_page);

    active_table.with(&mut new_page_table, &mut tmp_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("expected elf sections tag");

        for section in elf_sections_tag.sections() {

            if !section.is_allocated() {
                continue;
            }

            assert!(section.start_address() % PAGE_SIZE == 0,
                    "sections must be page aligned");

            println!("")

            for frame in Frame::range_inclusive(
                Frame::containing_address(section.start_address()),
                // end address is exclusive
                Frame::containing_address(section.end_address() - 1)
            ) {
                mapper.identity_map(frame, WRITABLE, allocator);
            }
        }
    });
}

use cga_screen::CGAScreen;
pub fn test_paging<A>(screen: &mut CGAScreen, page_table: &mut PageTable, allocator: &mut A)
    where A: FrameAllocator {

    println!(screen, "translate Some({}): {:?}", 0, page_table.translate(0));
    println!(screen, "translate Some({}): {:?}", 1024, page_table.translate(1024));
    println!(screen, "translate Some({}): {:?}", (1 << 30) - 1,
             page_table.translate((1 << 30) - 1));
    println!(screen, "translate ({}) None: {:?}", 1 << 30, page_table.translate(1 << 30));
    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_address(addr);
    let frame = allocator.alloc().expect("no more frames");
    println!(screen, "None = {:?}, map to {:?}",
             page_table.translate(addr),
             frame);
    page_table.map_to(page, frame.clone(), EntryFlags::empty(), allocator);
    println!(screen, "translate {}, Some = {:?}", addr, page_table.translate(addr));
    println!(screen, "next free frame: {:?}", allocator.alloc());
    // the following line should panic
    // page_table.map(Page::containing_address(addr), EntryFlags::empty(), allocator);

    let addr2 = addr + PAGE_SIZE;
    page_table.map(Page::containing_address(addr2), EntryFlags::empty(), allocator);
    println!(screen, "translate {}, Some = {:?}", addr2, page_table.translate(addr2));
    let addr3 = addr2 + PAGE_SIZE;
    page_table.identity_map(Frame::containing_address(addr3), EntryFlags::empty(), allocator);
    println!(screen, "translate {}, Some = {:?}", addr3, page_table.translate(addr3));

    // map to the same frame
    let addr4 = addr3 + PAGE_SIZE;
    page_table.map_to(Page::containing_address(addr4), frame, EntryFlags::empty(), allocator);
    println!(screen, "{:?} = {:?}", page_table.translate(addr), page_table.translate(addr4));
    unsafe {*(addr as *mut _) = 42;}
    println!(screen, "read 42: {}", unsafe {*(addr4 as *const usize)});
    page_table.unmap(Page::containing_address(addr4), allocator);
    // the following line should cause a page fault
    //println!(screen, "read 42: {}", unsafe {*(addr4 as *const usize)});
}
