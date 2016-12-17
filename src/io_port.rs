
pub struct IOPort {
    address: u16
}

impl IOPort {
    pub const fn new(a: u16) -> IOPort { IOPort { address: a } }

    pub fn outb(&self, val: u8) {
        unsafe {
            asm!("out %al, %dx"
                 :
                 :"{al}"(val), "{dx}"(self.address)
                 :);
        }
    }

    pub fn outw(&self, val: u16) {
        unsafe {
            asm!("outw %ax, %dx"
                 :
                 :"{al}"(val), "{dx}"(self.address)
                 :);
        }
    }

    pub fn inb(&self) -> u8 {
        let result: u8;
        unsafe {
            asm!("in %dx, %al"
                 :"={al}"(result)
                 :"{dx}"(self.address)
                 :);
        }
        result
    }
}
