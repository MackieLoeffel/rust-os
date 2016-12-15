#![feature(lang_items, const_fn)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod cga_screen;

use core::fmt;
use core::fmt::Write;

#[no_mangle]
pub extern fn rust_main() {

    cga_screen::clear();

    for i in 0..100 {
        println!("Hallo {}!", i);
        // write!(&mut screen, "Hallo {}!\n", i).unwrap();
    }

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
}

struct AssertWriter {
    line: u64, pos: u64
}

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
                    unsafe{ *(out as *mut _) = 0x7 << 4 | 0x4; }

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
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
