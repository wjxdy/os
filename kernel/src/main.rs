#![no_std]
#![no_main]

mod font;
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
    writeln!(serial, "framebuffer: text RUST OS drawn").expect("failed to write to COM1");
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

    let background = Color::new(24, 32, 56);
    let mut screen = FrameBufferWriter::new(framebuffer, 4, Color::WHITE, background);
    screen.clear_text_screen();

    writeln!(screen, "RUST OS").expect("failed to write title to framebuffer");
    writeln!(screen, "STATUS").expect("failed to write status to framebuffer");

    let project = "RUST";
    let system = "OS";
    writeln!(screen, "{project} {system}").expect("failed to write formatted text to framebuffer");
    writeln!(screen, "RUST OS RUST OS RUST OS RUST OS RUST OS")
        .expect("failed to write wrapping demo to framebuffer");

    let footer_scale = 2;
    let footer_height = 8usize.saturating_mul(footer_scale);
    let footer_y = info.height.saturating_sub(footer_height).saturating_sub(4);
    screen.draw_text(
        0,
        footer_y,
        "RUST OS",
        footer_scale,
        Color::new(80, 200, 120),
        background,
    );

    screen.draw_horizontal_line(
        0,
        info.height.saturating_sub(2),
        info.width,
        Color::new(80, 200, 120),
    );

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
