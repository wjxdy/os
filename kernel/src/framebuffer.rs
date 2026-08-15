use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

#[derive(Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const WHITE: Self = Self::new(255, 255, 255);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

pub struct FrameBufferWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: FrameBuffer) -> Self {
        let info = framebuffer.info();
        let buffer = framebuffer.into_buffer();
        Self { buffer, info }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let pixel_offset = (y * self.info.stride + x) * bytes_per_pixel;
        if pixel_offset + bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb if bytes_per_pixel >= 3 => {
                self.write_byte(pixel_offset, color.red);
                self.write_byte(pixel_offset + 1, color.green);
                self.write_byte(pixel_offset + 2, color.blue);
            }
            PixelFormat::Bgr if bytes_per_pixel >= 3 => {
                self.write_byte(pixel_offset, color.blue);
                self.write_byte(pixel_offset + 1, color.green);
                self.write_byte(pixel_offset + 2, color.red);
            }
            PixelFormat::U8 if bytes_per_pixel >= 1 => {
                let gray = ((color.red as u16 + color.green as u16 + color.blue as u16) / 3) as u8;
                self.write_byte(pixel_offset, gray);
            }
            _ => return,
        }
    }

    fn write_byte(&mut self, index: usize, value: u8) {
        unsafe {
            core::ptr::write_volatile(self.buffer.as_mut_ptr().add(index), value);
        }
    }

    pub fn clear(&mut self, color: Color) {
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn draw_horizontal_line(&mut self, x: usize, y: usize, length: usize, color: Color) {
        let x_end = x.saturating_add(length).min(self.info.width);
        for current_x in x..x_end {
            self.write_pixel(current_x, y, color);
        }
    }
}
