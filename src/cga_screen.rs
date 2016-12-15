use core::fmt;
use spin::Mutex;

#[allow(dead_code)]
pub enum Color {
      Black = 0, Blue, Green, Cyan,
      Red, Magenta, Brown, LightGrey,
      DarkGrey, LightBlue, LightGreen, LightCyan,
      LightRed, LightMagenta, Yellow, White
}
const STD_ATTR: u8 = ((Color::Black as u8 & 0x7) << 4) | (Color::LightGrey as u8 & 0xf);

const CGA_START: u64 = 0xb8000;
pub const COLUMNS: u64 = 80;
pub const ROWS: u64 = 25;

pub static SCREEN: Mutex<CGAScreen> = Mutex::new(CGAScreen::new_const(0, 0, COLUMNS, ROWS));

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::cga_screen::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    SCREEN.lock().write_fmt(args).unwrap();
}

pub fn clear() {
    SCREEN.lock().clear();
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
}

impl fmt::Write for CGAScreen {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}
