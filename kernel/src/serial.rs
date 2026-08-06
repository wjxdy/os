use core::arch::asm;

/// Minimal COM1 (UART 16550) driver.
pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    /// Configure the UART: disable interrupts, 38400 baud, 8N1, enable FIFO.
    pub fn init(&mut self) {
        // Disable interrupts (IER = 0).
        self.outb(1, 0x00);
        // Set DLAB so the next two writes configure the baud divisor.
        self.outb(3, 0x80);
        // Divisor 3 -> 38400 baud on a 1.8432 MHz clock.
        self.outb(0, 0x03);
        self.outb(1, 0x00);
        // Clear DLAB; 8 data bits, no parity, 1 stop bit (LCR = 0x03).
        self.outb(3, 0x03);
        // Enable and clear the FIFO (FCR = 0xC7).
        self.outb(2, 0xC7);
        // RTS/DSR + IRQ enabled (MCR = 0x0B).
        self.outb(4, 0x0B);
    }

    fn outb(&self, offset: u16, value: u8) {
        let port = self.base + offset;
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") value);
        }
    }

    fn inb(&self, offset: u16) -> u8 {
        let port = self.base + offset;
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") port);
        }
        value
    }

    fn write_byte(&mut self, byte: u8) {
        // Wait until the transmit holding register is empty (LSR bit 5).
        while self.inb(5) & 0x20 == 0 {}
        self.outb(0, byte);
    }
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
