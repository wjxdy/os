use core::arch::asm;

fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}

fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port);
    }
    value
}

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    pub const unsafe fn new(base: u16) -> Self {
        Self { base }
    }

    const fn data(&self) -> u16 {
        self.base
    }
    const fn ier(&self) -> u16 {
        self.base + 2
    }
    const fn fcr(&self) -> u16 {
        self.base + 3
    }
    const fn lcr(&self) -> u16 {
        self.base + 4
    }
    const fn lsr(&self) -> u16 {
        self.base + 5
    }

    pub fn init(&mut self) {
        outb(self.ier(), 0x00);
        outb(self.lcr(), 0x83);

        outb(self.data(), 0x03);
        outb(self.ier(), 0x00);

        outb(self.lcr(), 0x03);
        outb(self.fcr(), 0x07)
    }

    fn tx_ready(&self) -> bool {
        (inb(self.lsr()) & 0x20) != 0
    }

    pub fn send_raw(&mut self, byte: u8) {
        while !self.tx_ready() {
            core::hint::spin_loop();
        }
        outb(self.data(), byte);
    }
}
impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.send_raw(byte);
        }
        Ok(())
    }
}
