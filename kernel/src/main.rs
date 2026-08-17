#![no_std]
#![no_main]

mod framebuffer;
mod serial;
use bootloader_api::{BootInfo, entry_point};
use core::{fmt::Write, panic::PanicInfo};
use framebuffer::{Color, FrameBufferWriter};

entry_point!(kernel_main);

fn serial_port() -> serial::SerialPort {
    let mut port = unsafe { serial::SerialPort::new(0x3F8) };
    port.init();
    port
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial_port();
    writeln!(serial, "Rust OS: kernel entered").expect("failed to write to COM1");
    let framebuffer = boot_info
        .framebuffer
        .take()
        .expect("bootloader did not provide a framebuffer");
    let info = framebuffer.info();

    writeln!(
        serial,
        "framebuffer: {}x{}, stride={}, bytes_per_pixel={}, pixel_format={:?}",
        info.width, info.height, info.stride, info.bytes_per_pixel, info.pixel_format
    )
    .expect("failed to write framebuffer info to COM1");

    let mut screen = FrameBufferWriter::new(framebuffer);
    screen.clear(Color::new(24, 32, 56));
    screen.fill_rect(80, 80, 320, 180, Color::new(220, 50, 47));
    screen.draw_horizontal_line(80, 300, 500, Color::new(80, 200, 120));
    screen.write_pixel(info.width / 2, info.height / 2, Color::WHITE);

    writeln!(serial, "framebuffer: test pattern drawn")
        .expect("failed to write framebuffer status to COM1");

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
