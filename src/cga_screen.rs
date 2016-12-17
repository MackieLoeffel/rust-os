use core::fmt;
use spin::Mutex;

#[allow(dead_code)]
pub enum Color {
    Black = 0, Blue, Green, Cyan,
    Red, Magenta, Brown, LightGrey,
    Darkgrey, Lightblue, Lightgreen, Lightcyan,
    Lightred, Lightmagenta, Yellow, White,
}
const STD_ATTR: u8 = build_color(Color::LightGrey, Color::Black);

const CGA_START: u64 = 0xb8000;
pub const COLUMNS: u64 = 80;
pub const ROWS: u64 = 25;

pub static DBG: Mutex<CGAScreen> = Mutex::new(CGAScreen::new_const(0, 0, COLUMNS, 2));
pub static SCREEN: Mutex<CGAScreen> = Mutex::new(CGAScreen::new_const(0, 2, COLUMNS, ROWS - 2));

macro_rules! println {
    ($screen:expr, $fmt:expr) => (print!($screen, concat!($fmt, "\n")));
    ($screen:expr, $fmt:expr, $($arg:tt)*) => (print!($screen, concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($screen:expr, $($arg:tt)*) => ({
        $screen.print(format_args!($($arg)*));
    });
}

macro_rules! dbg {
    ($($arg:tt)*) => ({
        $crate::cga_screen::dbg(format_args!($($arg)*));
    });
}

#[allow(dead_code)]
pub fn dbg(args: fmt::Arguments) {
    let mut d = DBG.lock();
    d.write_byte(b'\n');
    d.print(args);
}

pub struct CGAScreen {
    from_col: u64, from_row: u64,
    size_x: u64, size_y: u64,
    cursor_x: u64, cursor_y: u64,
    color: u8
}

impl CGAScreen {
    const fn new_const(from_col: u64, from_row: u64, size_x: u64, size_y: u64) -> CGAScreen {
        CGAScreen{from_col: from_col, from_row: from_row,
                  size_x: size_x, size_y: size_y,
                  cursor_x: 0, cursor_y: 0,
                  color: STD_ATTR}
    }

    #[allow(dead_code)]
    pub fn new(from_col: u64, from_row: u64, size_x: u64, size_y: u64) -> CGAScreen {
        assert!(from_col + size_x <= COLUMNS);
        assert!(from_row + size_y <= ROWS);
        CGAScreen::new_const(from_col, from_row, size_x, size_y)
    }

    pub fn show(&mut self, x: u64, y: u64, b: u8) {
        assert!(x < self.size_x);
        assert!(y < self.size_y);
        let offset = ((y + self.from_row) * COLUMNS + (x + self.from_col)) * 2;
        let addr = CGA_START + offset;
        unsafe {
            *(addr as *mut _) = b;
            *((addr + 1) as *mut _) = self.color;
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
        let old_color = self.color;
        for crow in 0..self.size_y {
            for ccol in 0..self.size_x {
                if crow < self.size_y - amount {
                    let (character, attr) = self.get(ccol, crow + amount);
                    self.color = attr;
                    self.show(ccol, crow, character);
                } else {
                    self.show(ccol, crow, b' ');
                }
            }
        }
        self.color = old_color;
    }

    pub fn set_pos(&mut self, x: u64, y: u64) {
        assert!(x < self.size_x);
        assert!(y < self.size_y);
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn clear(&mut self) {
        let y = self.size_y;
        self.scroll_down(y);
        self.set_pos(0, 0);
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        use core::fmt::Write;
        self.write_fmt(args).unwrap();
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = build_color(fg, bg);
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

const fn build_color(fg: Color, bg: Color) -> u8 {
    ((bg as u8 & 0x7) << 4) | (fg as u8 & 0xf)
}
