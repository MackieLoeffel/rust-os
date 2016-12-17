use io_port::IOPort;

pub fn shutdown() {
    // TODO only works for qemu, use general solution
    // see http://forum.osdev.org/viewtopic.php?t=16990 and http://forum.osdev.org/viewtopic.php?t=16990
    let port = IOPort::new(0xb004);
    port.outw(0 | 0x2000);
}
