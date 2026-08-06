use ore::arch::asm;

fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") prot, in("al") value);
    }
}

fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port);
    }
    value
}
