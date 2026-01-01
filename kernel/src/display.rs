use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    Pixel,
    pixelcolor::Rgb888,
    prelude::{DrawTarget, OriginDimensions, RgbColor},
};
use spin::Mutex;

pub static DISPLAY: Mutex<Option<FramebufferDisplay>> = Mutex::new(None);

pub fn init(info: FrameBufferInfo, buffer: &mut [u8]) {
    let bytes = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr(), buffer.len()) };

    let mut fb_driver = FramebufferDisplay::new(info, bytes);

    fb_driver.clear(Rgb888::BLACK);

    *DISPLAY.lock() = Some(fb_driver);
}

pub struct FramebufferDisplay {
    info: FrameBufferInfo,
    bytes: &'static mut [u8],
}

impl FramebufferDisplay {
    pub fn new(info: FrameBufferInfo, bytes: &'static mut [u8]) -> Self {
        Self { info, bytes }
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

    pub fn info(&self) -> &FrameBufferInfo {
        &self.info
    }
}

impl DrawTarget for FramebufferDisplay {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        match self.info.pixel_format {
            PixelFormat::Bgr => {
                for Pixel(pos, color) in pixels.into_iter() {
                    let x = pos.x as usize;
                    let y = pos.y as usize;

                    if x < self.info.width && y < self.info.height {
                        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;

                        unsafe {
                            *self.bytes.get_unchecked_mut(offset + 0) = color.b();
                            *self.bytes.get_unchecked_mut(offset + 1) = color.g();
                            *self.bytes.get_unchecked_mut(offset + 2) = color.r();
                        }
                    }
                }
            }

            PixelFormat::Rgb => {
                for Pixel(pos, color) in pixels.into_iter() {
                    let x = pos.x as usize;
                    let y = pos.y as usize;

                    if x < self.info.width && y < self.info.height {
                        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;

                        unsafe {
                            *self.bytes.get_unchecked_mut(offset + 0) = color.r();
                            *self.bytes.get_unchecked_mut(offset + 1) = color.g();
                            *self.bytes.get_unchecked_mut(offset + 2) = color.b();
                        }
                    }
                }
            }

            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                let r_offset = red_position as usize / 8;
                let g_offset = green_position as usize / 8;
                let b_offset = blue_position as usize / 8;

                for Pixel(pos, color) in pixels.into_iter() {
                    let x = pos.x as usize;
                    let y = pos.y as usize;

                    if x < self.info.width && y < self.info.height {
                        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;

                        unsafe {
                            *self.bytes.get_unchecked_mut(offset + r_offset) = color.r();
                            *self.bytes.get_unchecked_mut(offset + g_offset) = color.g();
                            *self.bytes.get_unchecked_mut(offset + b_offset) = color.b();
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }
}

impl OriginDimensions for FramebufferDisplay {
    fn size(&self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size {
            width: self.info.width.try_into().unwrap(),
            height: self.info.height.try_into().unwrap(),
        }
    }
}
