use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Point, RgbColor},
    text::{Baseline, Text},
};

pub struct DisplayConsole {
    cursor: Point,
    style: MonoTextStyle<'static, Rgb888>,
}

impl DisplayConsole {
    pub fn new_line(&mut self) {
        self.cursor.x = 0;
        self.cursor.y += self.style.font.character_size.height as i32;
    }

    pub fn write_char(&mut self, ch: char) {
        use crate::display::DISPLAY;

        let mut guard = DISPLAY.lock();
        let Some(display) = guard.as_mut() else {
            return;
        };

        if self.cursor.x + self.style.font.character_size.width as i32
            >= display.info().width as i32
        {
            self.new_line();
        }

        self.put_char(display, ch, self.cursor);

        self.cursor.x += self.style.font.character_size.width as i32;
    }

    pub fn put_char<D: DrawTarget<Color = Rgb888>>(&self, display: &mut D, ch: char, pos: Point) {
        let mut buf = [0u8; 4];
        let str = ch.encode_utf8(&mut buf);

        let _ = Text::with_baseline(str, pos, self.style, Baseline::Top).draw(display);
    }
}

impl Default for DisplayConsole {
    fn default() -> Self {
        Self {
            cursor: Point::zero(),
            style: MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE),
        }
    }
}

impl core::fmt::Write for DisplayConsole {
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
