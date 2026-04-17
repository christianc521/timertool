use embedded_graphics::{pixelcolor::{raw::RawU16, *}, prelude::{Point, RawData}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RGBa {
    color: Rgb565,
    alpha: u8
}

impl RGBa {
    pub const fn new(color: C, alpha: u8) -> Self {
        Self {color, alpha}
    }

    pub const fn color(&self) -> Rgb565 {
        self.color
    }

    pub const fn alpha(&self) -> u8 {
        self.alpha
    }

    #[inline(always)]
    fn blend(&self, background: Rgb565) -> Rgb565 {
        if self.alpha == 255 {
            return self.color;
        }
        if self.alpha == 0 {
            return background;
        }

        blend_rgb565(self.color, background, self.alpha)
    }
}

#[inline(always)]
fn blend_rgb565(source: Rgb565, destination: Rgb565, alpha: u8) -> Rgb565 {
    let a = alpha as u32;
    let src = RawU16::from(source).into_inner() as u32;
    let dest = RawU16::from(destination).into_inner() as u32;

    // Separate R+B (packed) and G channels
    let src_rb = src & 0xF81F;
    let dest_rb = dest & 0xF81F;
    let src_g = src & 0x07E0;
    let dest_g = dest & 0x07E0;

    // Blend R+B together (they won't interfere in a u32 space)
    // dest + ((src - dest)) * alpha) >> 8
    let rb = dest_rb + (((src_rb.wrapping_sub(dest_rb)).wrapping_mul(a)) >> 8);
    // Mask to valid R+B bits
    let rb = rb & 0xF81F;  

    // Now doing G channel on it's own
    // ain't nothing but a G thing
    let g = dest_g + (((src_g.wrapping_sub(dest_g)).wrapping_mul(a)) >> 8);
    let g = g & 0x07E0;  

    // Recombine
    let raw = (rb | g) as u16;
    Rgb565::from(RawU16::new(raw))
}

// ------------------------------------------------------------
// BufferData integration
// ------------------------------------------------------------

use crate::buffer_backend::BufferData;
use crate::constants::{DISPLAY_WIDTH, DISPLAY_HEIGHT, PIXEL_COUNT};

impl BufferData {
    // Rectangular (only) blending directly over a region in the framebuffer
    // No allocation - only modifies existing pixels inside the region
    pub fn blend_solid_region(&mut self, rect: &Rectangle, overlay: RGBa) {
        if overlay.alpha() == 0 {
            return
        }

        // Boundary check and validation
        let x0 = rect.top_left.x.max(0) as u32;
        let y0 = rect.top_left.y.max(0) as u32;
        let x1 = (x0 + rect.size.width).min(DISPLAY_WIDTH);
        let y1 = (y0 + rect.size.height).min(DISPLAY_HEIGHT);

        for y in y0..y1 {
            let row_start = (y * DISPLAY_WIDTH + x0) as usize;
            let row_end = (y * DISPLAY_WIDTH + x1) as usize;

            for index in row_start..row_end {
                let bg = self.buffer[index];
                self.buffer[index] = overlay.blend(bg);
            }
        }
    }

    // Pixel iteration blending:
    // used for sprites with transparent backgrounds and non-rectangular primitives
    pub fn blend_iter(
        &mut self,
        position: Point,
        width: u32,
        height: u32,
        pixels: impl Iterator<Item = (Rgb565, u8)>,
    ) {
        let x0 = position.x.max(0) as u32;
        let y0 = position.y.max(0) as u32;

        let mut px = 0u32;
        for (color, alpha) in pixels {
            let local_x = px % width;
            let local_y = px / width;
            px += 1;

            if local_y >= height {
                break;
            }
            
            let screen_x = x0 + local_x;
            let screen_y = y0 + local_y;

            if screen_x >= DISPLAY_WIDTH || screen_y >= DISPLAY_HEIGHT {
                continue;
            }

            if alpha == 0 {
                continue;
            }

            let index = (screen_y * DISPLAY_WIDTH + screen_x) as usize;

            if alpha == 255 {
                // Fully opaque - no need to blend
                self.buffer[index] = color;
            } else {
                let bg = self.buffer[index];
                self.buffer[index] = RGBa::new(color, alpha).blend(bg);
            }
        }


    }
}
