use crate::font::{GLYPH_QUESTION, glyph_for};
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
    cursor_x: usize,
    cursor_y: usize,
    origin_x: usize,
    origin_y: usize,
    scale: usize,
    foreground: Color,
    background: Color,
}

impl FrameBufferWriter {
    pub fn new(
        framebuffer: FrameBuffer,
        scale: usize,
        foreground: Color,
        background: Color,
    ) -> Self {
        let info = framebuffer.info();
        let buffer = framebuffer.into_buffer();
        Self {
            buffer,
            info,
            cursor_x: 0,
            cursor_y: 0,
            origin_x: 0,
            origin_y: 0,
            scale: scale.max(1),
            foreground,
            background,
        }
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
            _ => {}
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

    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        let x_end = x.saturating_add(width).min(self.info.width);
        let y_end = y.saturating_add(height).min(self.info.height);

        for current_y in y..y_end {
            for current_x in x..x_end {
                self.write_pixel(current_x, current_y, color);
            }
        }
    }
    pub fn draw_glyph_8x8(
        &mut self,
        x: usize,
        y: usize,
        glyph: &[u8; 8],
        scale: usize,
        foreground: Color,
        background: Color,
    ) {
        if scale == 0 {
            return;
        }

        for (row, row_bits) in glyph.iter().copied().enumerate() {
            for column in 0usize..8 {
                let mask = 1u8 << (7 - column);
                let color = if row_bits & mask != 0 {
                    foreground
                } else {
                    background
                };

                let pixel_x = x.saturating_add(column.saturating_mul(scale));
                let pixel_y = y.saturating_add(row.saturating_mul(scale));
                self.fill_rect(pixel_x, pixel_y, scale, scale, color);
            }
        }
    }
    pub fn draw_char(
        &mut self,
        x: usize,
        y: usize,
        character: char,
        scale: usize,
        foreground: Color,
        background: Color,
    ) {
        let glyph = glyph_for(character).unwrap_or(&GLYPH_QUESTION);
        self.draw_glyph_8x8(x, y, glyph, scale, foreground, background);
    }
    pub fn draw_text(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        scale: usize,
        foreground: Color,
        background: Color,
    ) {
        if scale == 0 {
            return;
        }
        let origin_x = x;
        let mut cursor_x = x;
        let mut cursor_y = y;
        let advance = 9usize.saturating_mul(scale);

        for character in text.chars() {
            if character == '\n' {
                cursor_x = origin_x;
                cursor_y = cursor_y.saturating_add(advance);
                continue;
            }

            self.draw_char(cursor_x, cursor_y, character, scale, foreground, background);
            cursor_x = cursor_x.saturating_add(advance);
        }
    }

    fn glyph_width(&self) -> usize {
        8usize.saturating_mul(self.scale)
    }

    fn glyph_height(&self) -> usize {
        8usize.saturating_mul(self.scale)
    }

    fn cell_width(&self) -> usize {
        9usize.saturating_mul(self.scale)
    }

    fn line_height(&self) -> usize {
        9usize.saturating_mul(self.scale)
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_x = self.origin_x;
        self.cursor_y = self.origin_y;
    }

    pub fn clear_text_screen(&mut self) {
        self.clear(self.background);
        self.reset_cursor();
    }

    fn new_line(&mut self) {
        self.cursor_x = self.origin_x;
        let next_y = self.cursor_y.saturating_add(self.line_height());

        if next_y.saturating_add(self.glyph_height()) > self.info.height {
            self.clear_text_screen();
        } else {
            self.cursor_y = next_y;
        }
    }

    pub fn write_character(&mut self, character: char) {
        match character {
            '\n' => {
                self.new_line();
                return;
            }
            '\r' => {
                self.cursor_x = self.origin_x;
                return;
            }
            _ => {}
        }

        if self.cursor_x.saturating_add(self.glyph_width()) > self.info.width {
            self.new_line();
        }

        self.draw_char(
            self.cursor_x,
            self.cursor_y,
            character,
            self.scale,
            self.foreground,
            self.background,
        );

        self.cursor_x = self.cursor_x.saturating_add(self.cell_width());
    }
}

impl core::fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for character in text.chars() {
            self.write_character(character);
        }
        Ok(())
    }
}
