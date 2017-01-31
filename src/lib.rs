#![feature(lang_items, const_fn, asm, unique)]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86;

#[macro_use]
mod cga_screen;
mod io_port;
mod keyboard;
mod power;
mod misc;
mod memory;

use cga_screen::{SCREEN, CGAScreen, ROWS, COLUMNS};
use keyboard::{KEYBOARD};
use misc::windows;
use memory::FrameAllocator;
use memory::PAGE_TABLE;

use core::fmt;
use core::fmt::Write;

#[no_mangle]
pub extern fn rust_main(multiboot_info_address: usize) {
    let multiboot_info = unsafe {multiboot2::load(multiboot_info_address)};

    {
        let mut whole_screen = CGAScreen::new(0, 0, COLUMNS, ROWS);
        whole_screen.clear();
    }

    let mut screen = SCREEN.lock();
    let mut keyboard = KEYBOARD.lock();
    let mut page_table = PAGE_TABLE.lock();

    keyboard.init();
    screen.clear();

    let memory_map_tag = multiboot_info.memory_map_tag()
        .expect("expected memory map tag");
    println!(screen, "memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!(screen, "    start {:#x}, length: {:#x}",
                 area.base_addr, area.length);
    }
    let elf_sections_tag = multiboot_info.elf_sections_tag()
        .expect("Elf-sections tag required");
    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();
    let multiboot_start = multiboot_info_address;
    let multiboot_end = multiboot_info_address + (multiboot_info.total_size as usize);

    println!(screen, "kernel: start {:#x}, end: {:#x}", kernel_start, kernel_end);
    println!(screen, "multiboot: start {:#x}, end: {:#x}", multiboot_start, multiboot_end);
    let mut allocator = memory::RangeAllocator::new(memory_map_tag.memory_areas(),
                                            kernel_start as usize, kernel_end as usize,
                                            multiboot_start, multiboot_end);

    let new_frame = allocator.alloc().unwrap();
    println!(screen, "First Frame: {:?}", new_frame);

    memory::test_paging(&mut screen, &mut page_table, &mut allocator);

    loop {
        let key = keyboard.key_hit();
        if !key.valid() {continue;}
        println!(screen, "Key {}", key.ascii());

        if key.ascii() == 'q' {
            power::shutdown();
            windows();
        }
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() { }

struct AssertWriter { line: u64, pos: u64 }

impl fmt::Write for AssertWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        const NEXT_LINE: u64 = 80 * 2;
        const ASSERT_OUT: u64 = 0xb8000 + 1942;

        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    self.line += 1;
                    self.pos = 0;
                },
                _ => {
                    let mut out = ASSERT_OUT + self.pos * 2 + self.line * NEXT_LINE;
                    unsafe{ *(out as *mut _) = byte; }
                    out += 1;
                    unsafe{ *(out as *mut _) = 0x1 << 4 | 0xf; }

                    self.pos += 1;
                }
            }
        }
        Ok(())
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {

    let mut out = AssertWriter {line: 0, pos: 0};
    out.write_fmt(_msg).unwrap();
    write!(&mut out, "\nFile: {}\nLine: {}\n", _file, _line).unwrap();

    // hang
    loop{}
}


#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! { loop {} }
