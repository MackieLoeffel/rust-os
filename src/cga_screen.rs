use core::fmt;
use spin::Mutex;

#[allow(dead_code)]
mod color {
    pub const BLACK: u8 = 0;
    pub const BLUE: u8 = 1;
    pub const GREEN: u8 = 2;
    pub const CYAN: u8 = 3;
    pub const RED: u8 = 4;
    pub const MAGENTA: u8 = 5;
    pub const BROWN: u8 = 6;
    pub const LIGHT_GREY: u8 = 7;
    pub const DARKGREY: u8 = 8;
    pub const LIGHTBLUE: u8 = 9;
    pub const LIGHTGREEN: u8 = 10;
    pub const LIGHTCYAN: u8 = 11;
    pub const LIGHTRED: u8 = 12;
    pub const LIGHTMAGENTA: u8 = 13;
    pub const YELLOW: u8 = 14;
    pub const WHITE: u8 = 15;
}
const STD_ATTR: u8 = ((color::BLACK & 0x7) << 4) | (color::LIGHT_GREY & 0xf);

const CGA_START: u64 = 0xb8000;
pub const COLUMNS: u64 = 80;
pub const ROWS: u64 = 25;

pub static SCREEN: Mutex<CGAScreen> = Mutex::new(CGAScreen::new_const(0, 0, COLUMNS, ROWS));

macro_rules! println {
    ($screen:expr, $fmt:expr) => (print!($screen, concat!($fmt, "\n")));
    ($screen:expr, $fmt:expr, $($arg:tt)*) => (print!($screen, concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($screen:expr, $($arg:tt)*) => ({
        $screen.print(format_args!($($arg)*));
    });
}

pub struct CGAScreen {
    from_col: u64, from_row: u64,
    size_x: u64, size_y: u64,
    cursor_x: u64, cursor_y: u64
}

impl CGAScreen {
    const fn new_const(from_col: u64, from_row: u64, size_x: u64, size_y: u64) -> CGAScreen {
        // assert!(from_col + size_x <= COLUMNS);
        // assert!(from_row + size_y <= ROWS);

        CGAScreen{from_col: from_col, from_row: from_row,
                  size_x: size_x, size_y: size_y,
                  cursor_x: 0, cursor_y: 0}
    }

    #[allow(dead_code)]
    pub fn new(from_col: u64, from_row: u64, size_x: u64, size_y: u64) -> CGAScreen {
        assert!(from_col + size_x <= COLUMNS);
        assert!(from_row + size_y <= ROWS);
        CGAScreen::new_const(from_col, from_row, size_x, size_y)
    }

    pub fn show(&mut self, x: u64, y: u64, b: u8) {
        self.show_attr(x, y, b, STD_ATTR)
    }

    pub fn show_attr(&mut self, x: u64, y: u64, b: u8, attr: u8) {
        assert!(x < self.size_x);
        assert!(y < self.size_y);
        let offset = ((y + self.from_row) * COLUMNS + (x + self.from_col)) * 2;
        let addr = CGA_START + offset;
        unsafe {
            *(addr as *mut _) = b;
            *((addr + 1) as *mut _) = attr;
        }
    }

    pub fn get(&self, x: u64, y: u64) -> (u8, u8) {
        assert!(x < self.size_x);
        assert!(y < self.size_y);
        let offset = ((y + self.from_row) * COLUMNS + (x + self.from_col)) * 2;
        let addr = CGA_START + offset;
        unsafe {
            (*(addr as *mut _), *((addr + 1) as *mut _))
        }

    }

    pub fn write_byte(&mut self, b: u8) {

        if b != b'\n' {
            let x = self.cursor_x;
            let y = self.cursor_y;
            self.show(x, y, b);
            self.cursor_x += 1;
        }

        if b == b'\n' || self.cursor_x == self.size_x {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }

        if self.cursor_y == self.size_y {
            self.scroll_down(1);
            self.cursor_y -= 1;
            self.cursor_x = 0;
        }
    }

    pub fn scroll_down(&mut self, amount: u64) {
        for crow in 0..self.size_y {
            for ccol in 0..self.size_x {
                if crow < self.size_y - amount {
                    let (character, attr) = self.get(ccol, crow + amount);
                    self.show_attr(ccol, crow, character, attr);
                } else {
                    self.show(ccol, crow, b' ');
                }
            }
        }
    }

    pub fn clear(&mut self) {
        let y = self.size_y;
        self.scroll_down(y);
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        use core::fmt::Write;
        self.write_fmt(args).unwrap();
    }
}

impl fmt::Write for CGAScreen {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}
