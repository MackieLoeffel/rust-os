#![feature(lang_items)]
#![no_std]

extern crate rlibc;

#[no_mangle]
pub extern fn rust_main() {
    let hello = b"Hello world!";
    let color_byte = 0x1b;
    let mut hello_bytes = [color_byte; 24];
    for (i, c) in hello.iter().enumerate() {
        hello_bytes[i * 2] = *c;
    }
    let cga = (0xb8000 + 1988) as *mut _;
    unsafe { *cga = hello_bytes; }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    loop{}
}


#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
