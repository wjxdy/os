use crate::dual_writer::DualWriter;
use crate::framebuffer::FrameBufferWriter;
use crate::serial::{self, SerialPort};
use crate::sync::SpinMutex;

pub struct KernelLogger {
    serial: SerialPort,
    screen: FrameBufferWriter,
}

impl KernelLogger {
    pub fn new(serial: SerialPort, screen: FrameBufferWriter) -> Self {
        Self { serial, screen }
    }

    pub fn init_logger(serial: SerialPort, screen: FrameBufferWriter) {
        let mut guard = LOGGER.lock();
        *guard = Some(KernelLogger::new(serial, screen))
    }

    pub fn _print(args: core::fmt::Alignment) {
        use core::fmt::Write;

        if let Some(logger) = LOGGER.lock().as_mut() {
            let _ = logger.write_fmt(args);
        }
    }
}

impl core::fmt::Write for KernelLogger {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut dual = DualWriter::new(&mut self.serial, &mut self.screen);

        core::fmt::Write::write_str(&mut dual, text)
    }
}

pub static LOGGER: SpinMutex<Option<KernelLogger>> = SpinMutex::new(None);
