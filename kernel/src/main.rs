#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod dual_writer;
mod font;
mod framebuffer;
mod interrupts;
pub mod logger;
mod memory;
mod serial;
mod sync;
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::{fmt::Write, panic::PanicInfo};
use framebuffer::{Color, FrameBufferWriter};
use memory::BootInfoFrameAllocator;
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::FrameAllocator;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

pub(crate) fn serial_port() -> serial::SerialPort {
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

    let background = Color::new(24, 32, 56);
    let mut screen = FrameBufferWriter::new(framebuffer, 4, Color::WHITE, background);
    screen.clear_text_screen();

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

    logger::init_logger(serial, screen);

    interrupts::init();

    let physical_memory_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("bootloader did not map physical memory"),
    );
    println!("PHYS OFFSET: {:#x}", physical_memory_offset.as_u64());

    let (p4_frame, _) = Cr3::read();
    let p4_phys = p4_frame.start_address();
    println!("P4 PHYS: {:#x}", p4_phys.as_u64());
    let p4_virt = physical_memory_offset + p4_phys.as_u64();
    println!("P4 VIRT: {:#x}", p4_virt.as_u64());

    let mapper = unsafe { memory::init_offset_page_table(physical_memory_offset) };
    let kernel_address = VirtAddr::new(kernel_main as *const () as u64);
    memory::print_level_4_entry(&mapper, kernel_address);

    let stack_address = VirtAddr::from_ptr(&physical_memory_offset);
    let boot_info_address = VirtAddr::from_ptr(&*boot_info);

    memory::print_address_pasts("KERNEL", kernel_address);
    memory::print_translation(&mapper, "STARK", stack_address);
    memory::print_translation(&mapper, "BOOT INFO", boot_info_address);
    memory::print_translation(&mapper, "PHYSICAL ZERO WINDOW", physical_memory_offset);

    memory::print_memory_map(&boot_info.memory_regions);

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    for index in 0..5 {
        match frame_allocator.allocate_frame() {
            Some(frame) => crate::println!(
                "ALLOCATED FRAME {index}: {:#x}",
                frame.start_address().as_u64()
            ),
            None => crate::println!("ALLOCATED FRAME {index}: NONE"),
        }
    }

    println!("RUST OS");

    println!("STATUS");

    let project = "RUST";
    let system = "OS";
    println!("{project} {system}");

    println!("RUST OS RUST OS RUST OS RUST OS RUST OS");

    println!("RUST OS OK");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if !logger::try_print_panic(format_args!("KERNEL PANIC: {info}\n")) {
        let _ = writeln!(serial_port(), "EMERGENCY KERNEL PANIC: {info}");
    };

    loop {
        core::hint::spin_loop();
    }
}
