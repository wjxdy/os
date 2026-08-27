use crate::{framebuffer::FrameBufferWriter, serial::SerialPort};

pub struct DualWriter<'a> {
    serial: &'a mut SerialPort,
    screen: &'a mut FrameBufferWriter,
}

impl<'a> DualWriter<'a> {
    pub fn new(serial: &'a mut SerialPort, screen: &'a mut FrameBufferWriter) -> Self {
        Self { serial, screen }
    }
}

impl<'a> core::fmt::Write for DualWriter<'a> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let serial_result = core::fmt::Write::write_str(&mut *self.serial, text);
        let screen_result = core::fmt::Write::write_str(&mut *self.screen, text);

        serial_result.and(screen_result)
    }
}
