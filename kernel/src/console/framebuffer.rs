use bootloader_api::info::{FrameBufferInfo, PixelFormat};

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 8;

const BACKGROUND_COLOR: [u8; 3] = [0x0, 0x0, 0x0];
const FOREGROUND_COLOR: [u8; 3] = [0xff, 0xff, 0xff];

pub struct FramebufferDriver {
    info: FrameBufferInfo,
    bytes: &'static mut [u8],

    x_pos: usize,
    y_pos: usize,
}

impl FramebufferDriver {
    pub fn new(info: FrameBufferInfo, bytes: &'static mut [u8]) -> Self {
        Self {
            info,
            bytes,

            x_pos: 0,
            y_pos: 0,
        }
    }

    pub fn put_pixel(&mut self, w: usize, h: usize, color: [u8; 3]) {
        let index = ((h * self.info.stride) + w) * self.info.bytes_per_pixel;

        match self.info.pixel_format {
            PixelFormat::Bgr => {
                self.bytes[index + 0] = color[2];
                self.bytes[index + 1] = color[1];
                self.bytes[index + 2] = color[0];
            }

            PixelFormat::Rgb => {
                self.bytes[index + 0] = color[0];
                self.bytes[index + 1] = color[1];
                self.bytes[index + 2] = color[2];
            }

            PixelFormat::U8 => {
                self.bytes[index] = color[0] + color[1] + color[2];
            }

            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                self.bytes[index + red_position as usize] = color[0];
                self.bytes[index + green_position as usize] = color[1];
                self.bytes[index + blue_position as usize] = color[2];
            }

            _ => {}
        }
    }

    pub fn put_char(&mut self, w: usize, h: usize, ch: char) {
        // TODO: Optimize me

        let glyph = match font8x8::legacy::BASIC_LEGACY.get(ch as usize) {
            Some(glyph) => glyph,
            None => return,
        };

        for (row_idx, row_data) in glyph.iter().enumerate() {
            for col_idx in 0..FONT_WIDTH {
                if (row_data >> col_idx) & 1 == 1 {
                    self.put_pixel(w + col_idx, h + row_idx, FOREGROUND_COLOR);
                } else {
                    self.put_pixel(w + col_idx, h + row_idx, BACKGROUND_COLOR);
                }
            }
        }
    }

    pub fn new_line(&mut self) {
        self.x_pos = 0;
        self.y_pos += FONT_HEIGHT;
    }

    pub fn clear_screen(&mut self) {
        // TODO: Optimize me

        for h in 0..self.info.height {
            for w in 0..self.info.width {
                self.put_pixel(w, h, BACKGROUND_COLOR);
            }
        }
    }

    pub fn write_char(&mut self, ch: char) {
        if self.x_pos + FONT_WIDTH >= self.info.width {
            self.new_line();
        }

        self.put_char(self.x_pos, self.y_pos, ch);

        self.x_pos += FONT_WIDTH;
    }
}

impl core::fmt::Write for FramebufferDriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            match ch {
                '\n' => self.new_line(),
                _ => self.write_char(ch),
            }
        }

        Ok(())
    }
}
