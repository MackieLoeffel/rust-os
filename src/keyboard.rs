use io_port::{IOPort};
use spin::Mutex;
use power;

static CTRL_PORT: IOPort = IOPort::new(0x64);
static DATA_PORT: IOPort = IOPort::new(0x60);
const OUTB: u8 = 0x01;
const INPB: u8 = 0x02;
const AUXB: u8 = 0x20;
const SET_SPEED: u8 = 0xf3;
const BREAK_BIT: u8 = 0x80;
const PREFIX1: u8 = 0xe0;
const PREFIX2: u8 = 0xe1;
static NORMAL_TAB: [u8; 89] = [
    0, 0, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', 225, 39, 0,
    0, b'q', b'w', b'e', b'r', b't', b'z', b'u', b'i', b'o', b'p', 129, b'+', b'\n',
    0, b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', 148, 132, b'^', 0, b'#',
    b'y', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'-', 0,
    b'*', 0, b' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'-',
    0, 0, 0, b'+', 0, 0, 0, 0, 0, 0, 0, b'<', 0, 0
];

fn hooks(code: u8) {
    // low level keyboard hooks
    match code {
        1 => power::shutdown(),
        _ => {}
    }
}

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {initialized: false, prefix: 0, gather: Key::invalid()});

pub struct Keyboard {
    initialized: bool,
    gather: Key,
    prefix: u8
}

impl Keyboard {

    pub fn init(&mut self) {
        assert!(!self.initialized);

        self.drain_keyboard_buffer();
        self.set_repeat_rate(0, 0);

        self.initialized = true;
    }

    pub fn key_hit(&mut self) -> Key {
        assert!(self.initialized);

        let status = CTRL_PORT.inb();
        if (status & OUTB) != 0 {
            // key ready to be read
            let code = DATA_PORT.inb();
            if (status & AUXB) == 0 {
                // valus is from keyboard, not from mouse
                if self.key_decoded(code) {
                    return self.gather;
                }
            }
        }

        Key::invalid()
    }

    fn key_decoded(&mut self, code: u8) -> bool {
        if code == PREFIX1 || code == PREFIX2 {
            self.prefix = code;
            return false;
        }

        if (code & BREAK_BIT) != 0 {
            // TODO implement break bitB
            self.prefix = 0;
            return false;
        }

        let mut done = false;
        hooks(code);
        match code {
            42 | 54 => {} // TODO: shift
            56 => {} // TODO: alt left right
            29 => {} // TODO: ctrl left right
            58 => {} // TODO: capslock
            70 => {} // TODO: scroll lock
            69 => {} // TODO: numlock/pause
            _ => {
                self.compute_key(code);
                done = true;
            }
        }

        self.prefix = 0;
        done
    }

    fn compute_key(&mut self, code: u8) {
        if code == 53 && self.prefix == PREFIX1 {
            self.gather.set_ascii(b'/');
        } else {
            self.gather.set_ascii(NORMAL_TAB[code as usize]);
        }
    }

    fn drain_keyboard_buffer(&mut self) {
        while (CTRL_PORT.inb() & OUTB) != 0 {
            DATA_PORT.inb();
        }
    }

    fn set_repeat_rate(&mut self, speed: u8, delay: u8) {
        assert!(speed <= 3);
        assert!(delay <= 31);
        self.send_command(SET_SPEED, speed | (delay << 4));
    }

    fn send_command(&mut self, cmd: u8, data: u8) {
        self.send_byte(cmd);
        self.send_byte(data);
    }

    fn send_byte(&mut self, byte: u8) {
        while (CTRL_PORT.inb() & INPB) != 0 {}
        DATA_PORT.outb(byte);
    }
}


#[derive(Copy, Clone)]
pub struct Key {
    valid: bool,
    ascii: u8
}

impl Key {
    pub const fn invalid() -> Key { Key {valid: false, ascii: 0} }

    pub fn valid(&self) -> bool { self.valid }

    pub fn ascii(&self) -> char {
        assert!(self.valid);
        if self.ascii >= 128 {
            return '?';
        }
        self.ascii.into()
    }

    fn set_ascii(&mut self, ascii: u8) {
        self.valid = true;
        self.ascii = ascii;
    }
}
