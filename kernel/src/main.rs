#![no_std]
#![no_main]

mod serial;

use bootloader_api::{BootInfo, entry_point};
use core::{fmt::Write, panic::PanicInfo};

entry_point!(kernel_main);

const COM1: u16 = 0x3F8;

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial::SerialPort::new(COM1);
    serial.init();
    writeln!(serial, "Rust OS: kernel entered").expect("failed to write to COM1");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = serial::SerialPort::new(COM1);
    serial.init();
    let _ = writeln!(serial, "KERNEL PANIC: {info}");

    loop {
        core::hint::spin_loop();
    }
}
