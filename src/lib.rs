#![feature(lang_items, const_fn, asm)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod cga_screen;
mod io_port;
mod keyboard;

use cga_screen::{SCREEN, Color, CGAScreen, ROWS, COLUMNS};
use keyboard::{KEYBOARD};

use core::fmt;
use core::fmt::Write;

#[no_mangle]
pub extern fn rust_main() {

    let mut screen = SCREEN.lock();
    let mut keyboard = KEYBOARD.lock();

    keyboard.init();
    screen.clear();
    println!(screen, "Booted!");

    loop {
        let key = keyboard.key_hit();
        if !key.valid() {continue;}
        println!(screen, "Key {}", key.ascii());

        if key.ascii() == 'q' {
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

pub fn windows() {
    let mut screen = CGAScreen::new(0, 0, COLUMNS, ROWS);

    screen.set_color(Color::White, Color::Blue);
    screen.clear();
    println!(screen, "A problem has been detected and Windows has been shut down to prevent damage");
    println!(screen, "to your computer.");
    println!(screen, "");
    println!(screen, "The problem seems to be caused by the following file: SPCMDCON.SYS");
    println!(screen, "PAGE_FAULT_IN_NONPAGED_AREA");
    println!(screen, "If this is the first time you've seen this stop error screen,");
    println!(screen, "restart your computer. If this screen appears again, follow");
    println!(screen, "these steps:");
    println!(screen, "");
    println!(screen, "Check to make sure any new hardware or software is properly installed.");
    println!(screen, "If this is a new installation, ask your hardware or software manufacturer");
    println!(screen, "for any Windows updates you might need.");
    println!(screen, "");
    println!(screen, "If problems continue, disable or remove any newly installed hardware");
    println!(screen, "or software. Disable BIOS memory options such as caching or shadowing.");
    println!(screen, "If you need to use Safe Mode to remove or disable components, restart");
    println!(screen, "your computer, press F8 to select Advanced Startup Options, and then");
    println!(screen, "select Safe Mode.");
    println!(screen, "");
    println!(screen, "Technical information:");
    println!(screen, "");
    println!(screen, "*** STOP: 0x00000050 (0xFD3094C2,0x00000001,0xFBFE7617,0x00000000)");
    println!(screen, "");
    println!(screen, "***  SPCMDCON.SYS - Address FBFE7617 base at FBFE5000, DateStamp 3d6dd67c");

    loop {}
}
