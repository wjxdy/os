#![no_std]
#![no_main]

mod font;
mod framebuffer;
mod serial;
use bootloader_api::{BootInfo, entry_point};
use core::{fmt::Write, panic::PanicInfo};
use font::GLYPH_A;
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
    let background = Color::new(24, 32, 56);
    screen.clear(background);

    let scale = 12;
    let glyph_size = 8usize.saturating_mul(scale);
    let glyph_x = info.width.saturating_sub(glyph_size) / 2;
    let glyph_y = info.height.saturating_sub(glyph_size) / 2;
    screen.draw_glyph_8x8(glyph_x, glyph_y, &GLYPH_A, scale, Color::WHITE, background);
    let underline_y = glyph_y.saturating_add(glyph_size).saturating_add(12);
    screen.draw_horizontal_line(glyph_x, underline_y, glyph_size, Color::new(80, 200, 120));

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
