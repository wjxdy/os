use crate::dual_writer::DualWriter;
use crate::framebuffer::{Color, FrameBufferWriter};
use crate::serial::SerialPort;
use crate::sync::SpinMutex;

pub struct KernelLogger {
    serial: SerialPort,
    screen: FrameBufferWriter,
}

impl KernelLogger {
    pub fn new(serial: SerialPort, screen: FrameBufferWriter) -> Self {
        Self { serial, screen }
    }

    pub fn write_panic(&mut self, args: core::fmt::Arguments) -> core::fmt::Result {
        use core::fmt::Write;

        let panic_background = Color::new(96, 16, 24);

        self.screen.set_text_colors(Color::WHITE, panic_background);
        self.screen.clear_text_screen();
        self.write_fmt(args)
    }
}

impl core::fmt::Write for KernelLogger {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut dual = DualWriter::new(&mut self.serial, &mut self.screen);

        core::fmt::Write::write_str(&mut dual, text)
    }
}

pub static LOGGER: SpinMutex<Option<KernelLogger>> = SpinMutex::new(None);

pub fn init_logger(serial: SerialPort, screen: FrameBufferWriter) {
    let mut guard = LOGGER.lock();
    *guard = Some(KernelLogger::new(serial, screen))
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if let Some(logger) = LOGGER.lock().as_mut() {
        let _ = logger.write_fmt(args);
    }
}

pub fn try_print(args: core::fmt::Arguments) -> bool {
    use core::fmt::Write;

    let mut guard = match LOGGER.try_lock() {
        Some(guard) => guard,
        None => return false,
    };

    let logger = match guard.as_mut() {
        Some(logger) => logger,
        None => return false,
    };

    let _ = logger.write_fmt(args);

    true
}

pub fn try_print_panic(args: core::fmt::Arguments) -> bool {
    let mut guard = match LOGGER.try_lock() {
        Some(guard) => guard,
        None => return false,
    };

    let logger = match guard.as_mut() {
        Some(logger) => logger,
        None => return false,
    };

    let _ = logger.write_panic(args);
    true
}
