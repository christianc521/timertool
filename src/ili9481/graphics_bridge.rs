// ──────────────────────────────────────────────
// embedded-graphics integration
// ──────────────────────────────────────────────

use crate::ili9481::ili9481_driver::{Ili9481, Rgb565Mode};
use display_interface::WriteOnlyDataCommand;
use embedded_graphics::prelude::*;
use embedded_graphics::{ 
    pixelcolor::{Rgb565, raw::RawU16},
    primitives::Rectangle,
};
use embedded_hal::digital::OutputPin;

impl<DI, RST> OriginDimensions for Ili9481<DI, RST, Rgb565Mode> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl<DI, RST> DrawTarget for Ili9481<DI, RST, Rgb565Mode>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    type Color = Rgb565;
    type Error = display_interface::DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }
            self.draw_raw_slice(
                x as u16,
                y as u16,
                x as u16,
                y as u16,
                &[color],
            );
        }
        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // Clip to display bounds
        let display_area = Rectangle::new(Point::zero(), self.size());
        let clipped = area.intersection(&display_area);
        if clipped.size.width == 0 || clipped.size.height == 0 {
            return Ok(());
        }

        let x0 = clipped.top_left.x as u16;
        let y0 = clipped.top_left.y as u16;
        let x1 = x0 + clipped.size.width as u16 - 1;
        let y1 = y0 + clipped.size.height as u16 - 1;

        self.set_window(x0, y0, x1, y1);

        // Stream pixel data in chunks
        const CHUNK: usize = 170;
        let mut buf = [0u8; CHUNK * 3];
        let mut count = 0;

        // Calculate offsets for clipping
        let skip_left = (clipped.top_left.x - area.top_left.x) as usize;
        let skip_right = area.size.width as usize - clipped.size.width as usize - skip_left;
        let skip_top = (clipped.top_left.y - area.top_left.y) as usize;

        let mut iter = colors.into_iter();

        // Skip top rows
        for _ in 0..(skip_top * area.size.width as usize) {
            iter.next();
        }

        for _row in 0..clipped.size.height {
            // Skip left pixels
            for _ in 0..skip_left {
                iter.next();
            }

            // Visible pixels
            for _col in 0..clipped.size.width {
                if let Some(px) = iter.next() {
                    let raw = RawU16::from(px).into_inner();
                    let r5 = ((raw >> 11) & 0x1F) as u8;
                    let g6 = ((raw >> 5) & 0x3F) as u8;
                    let b5 = (raw & 0x1F) as u8;

                    buf[count * 3] = (r5 << 3) | (r5 >> 2);
                    buf[count * 3 + 1] = (g6 << 2) | (g6 >> 4);
                    buf[count * 3 + 2] = (b5 << 3) | (b5 >> 2);
                    count += 1;

                    if count == CHUNK {
                        self.write_data(&buf);
                        count = 0;
                    }
                }
            }

            // Skip right pixels
            for _ in 0..skip_right {
                iter.next();
            }
        }

        // Flush remaining
        if count > 0 {
            self.write_data(&buf[..count * 3]);
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let w = self.width;
        let h = self.height;
        self.set_window(0, 0, w - 1, h - 1);

        let raw: u16 = RawU16::from(color).into_inner();
        let r5 = ((raw >> 11) & 0x1F) as u8;
        let g6 = ((raw >> 5) & 0x3F) as u8;
        let b5 = (raw & 0x1F) as u8;
        let r = (r5 << 3) | (r5 >> 2);
        let g = (g6 << 2) | (g6 >> 4);
        let b = (b5 << 3) | (b5 >> 2);

        const CHUNK_PIXELS: usize = 170;
        let mut buf = [0u8; CHUNK_PIXELS * 3];
        for i in 0..CHUNK_PIXELS {
            buf[i * 3] = r;
            buf[i * 3 + 1] = g;
            buf[i * 3 + 2] = b;
        }

        let total_pixels = w as usize * h as usize;
        let full_chunks = total_pixels / CHUNK_PIXELS;
        let remainder = total_pixels % CHUNK_PIXELS;

        for _ in 0..full_chunks {
            self.write_data(&buf);
        }
        if remainder > 0 {
            self.write_data(&buf[..remainder * 3]);
        }

        Ok(())
    }
}
