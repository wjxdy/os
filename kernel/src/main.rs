#![no_std]
#![no_main]

mod serial;

use bootloader_api::{BootInfo, entry_point};
use core::{fmt::Write, panic::PanicInfo};

entry_point!(kernel_main);

fn serial_port() -> serial::SerialPort {
    let mut port = unsafe { serial::SerialPort::new(0x3F8) };
    port.init();
    port
}

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial_port();
    writeln!(serial, "Rust Os: kernel entered").expect("failed to write to COM1");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(serial_port(), "KERNEL PANIC: {info}");

    loop {
        core::hint::spin_loop();
    }
}
