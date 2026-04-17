use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use heapless::Vec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientDirection {
    Horizontal,
    Vertical
}

#[derive(Clone, Debug)]
pub struct Gradient {
    pub color_start: Rgb565,
    pub color_end: Rgb565,
    pub top_left: Point,
    pub rect_size: Size,
    pub direction: GradientDirection
}

impl Gradient {
    pub fn new(color_start: Rgb565, color_end: Rgb565) -> Self {
        Self {
            color_start,
            color_end,
            top_left: Point::zero(),
            rect_size: Size::zero(),
            direction: GradientDirection::Vertical
        }
    }
    pub fn direction(mut self, dir: GradientDirection) -> Self {
        self.direction = dir;
        self
    }

    pub fn position(mut self, top_left: Point) -> Self {
        self.top_left = top_left;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.rect_size = size;
        self
    }

    fn steps(&self) -> u32 {
        match self.direction {
            GradientDirection::Horizontal => self.rect_size.width,
            GradientDirection::Vertical => self.rect_size.height
        }
    }

    fn color_at(&self, step: u32) -> Rgb565 {
        let steps = self.steps();
        if steps <= 1 {
            return self.color_start;
        }

        let t = step as f32 / (steps - 1) as f32;

        // Normalize each color channel from 0-255 to 0-1
        let r1 = self.color_start.r() as f32 / 31.0;
        let g1 = self.color_start.g() as f32 / 63.0;
        let b1 = self.color_start.b() as f32 / 31.0;

        let r2 = self.color_end.r() as f32 / 31.0;
        let g2 = self.color_end.g() as f32 / 63.0;
        let b2 = self.color_end.b() as f32 / 31.0;

        // Inverse sRGB companding
        let linear_r1 = srgb_to_linear(r1);
        let linear_g1 = srgb_to_linear(g1);
        let linear_b1 = srgb_to_linear(b1);

        let linear_r2 = srgb_to_linear(r2);
        let linear_g2 = srgb_to_linear(g2);
        let linear_b2 = srgb_to_linear(b2);

        // Lerp in linear space
        let lerp_r = linear_r1 + (linear_r2 - linear_r1) * t;
        let lerp_g = linear_g1 + (linear_g2 - linear_g1) * t;
        let lerp_b = linear_b1 + (linear_b2 - linear_b1) * t;

        // Return back to sRGB and quantize
        let r = (linear_to_srgb(lerp_r) * 31.0 + 0.5) as u8;
        let g = (linear_to_srgb(lerp_g) * 63.0 + 0.5) as u8;
        let b = (linear_to_srgb(lerp_b) * 31.0 + 0.5) as u8;

        Rgb565::new(r.min(31), g.min(63), b.min(31))
    }
}

impl Drawable for Gradient {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: DrawTarget<Color = Self::Color> {
        let steps = self.steps();

        for i in 0..steps {
            let color = self.color_at(i);

            let (line_origin, line_size) = match self.direction {
                GradientDirection::Vertical => 
                    (
                        Point::new(
                            self.top_left.x,
                            self.top_left.y + i as i32
                            ),
                        Size::new(self.rect_size.width, 1),
                    ),
                GradientDirection::Horizontal =>
                    (
                        Point::new(
                            self.top_left.x + i as i32,
                            self.top_left.y
                            ),
                        Size::new(1, self.rect_size.height),
                    ),
            };

            let line = Rectangle::new(line_origin, line_size);
            target.fill_solid(&line, color)?;
        }
        Ok(())
    }
}

impl Dimensions for Gradient {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.rect_size)
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.0 {
        return 0.0;
    }
    if c >= 1.0 {
        return 1.0;
    }

    if c<= 0.04045 {
        c / 12.92
    } else {
        1.055 * pow_approx(( c + 0.055 ) / 1.055, 2.4) 
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0 {
        return 0.0;
    }
    if c >= 1.0 {
        return 1.0
    }
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * pow_approx(c, 1.0 / 2.4) - 0.055
    }
}

fn pow_approx(base: f32, exp: f32) -> f32 {
    // x^y = 2^(y * log2(x))
    let log2 = ln_approx(base) * core::f32::consts::LOG2_E;
    exp2_approx(exp * log2)
}

fn ln_approx(x: f32) -> f32 {
    // ln(x) = (as_int / 2^23 - 127) * ln(2)
    let bits = x.to_bits() as f32;
    let log2_approx = bits * 1.1920929e-7 - 126.94269504;
    log2_approx * core::f32::consts::LN_2
}

fn exp2_approx(x: f32) -> f32 {
    // 2^x by bit manipulation
    let clamped = x.max(-126.0).min(126.0);
    let bits = ((clamped + 126.94269504) * 8388608.0) as u32;
    f32::from_bits(bits)
}

fn split_rgb(color: Rgb565) -> (f32, f32, f32) {
    let red = color.r() as f32;
    let green = color.g() as f32;
    let blue = color.b() as f32;
    (red, green, blue)
}

fn normalize(color: Rgb565) -> (f32, f32, f32) {
    let (red, green, blue) = split_rgb(color);

    let normalized_red = red as f32 / 255.0;
    let normalized_green = green as f32 / 255.0;
    let normalized_blue = blue as f32 / 255.0;

    (normalized_red, normalized_green, normalized_blue)
}

fn mix(color_1: Rgb565, color_2: Rgb565) -> (f32, f32, f32) {
    let red_1 = color_1.r() as f32;
    let red_2 = color_2.r() as f32;
    let green_1 = color_1.r() as f32;
    let green_2 = color_2.g() as f32;
    let blue_1 = color_1.b() as f32;
    let blue_2 = color_2.b() as f32;

    let mix_red = (red_1 + red_2) / 2.0;
    let mix_green = (green_1 + green_2) / 2.0;
    let mix_blue = (blue_1 + blue_2) / 2.0;

    (mix_red, mix_green, mix_blue)
}

fn inverse_srgb_companding(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let mut r = micromath::F32::from(r);
    let mut g = micromath::F32::from(g);
    let mut b = micromath::F32::from(b);
    if r > 0.04045 {
        r = ((r + 0.055) / 1.055).powf(micromath::F32::from(2.4));
    } else {
        r /= micromath::F32::from(12.92)
    }

    if g > 0.04045 {
        g = ((g + 0.055) / 1.055).powf(micromath::F32::from(2.4));
    } else {
        g /= micromath::F32::from(12.92)
    }

    if b > 0.04045 {
        b = ((b + 0.055) / 1.055).powf(micromath::F32::from(2.4));
    } else {
        b /= micromath::F32::from(12.92)
    }
    (r.into(), g.into(), b.into())
}

fn srgb_companding(r: f32, g: f32, b: f32) -> Rgb565 {
    // Normalize channels from 0-255 to 0-1
    let normal_r = r / 255.0;
    let normal_g = g / 255.0;
    let normal_b = b / 255.0;
    let mut normal_r = micromath::F32::from(normal_r);
    let mut normal_g = micromath::F32::from(normal_g);
    let mut normal_b = micromath::F32::from(normal_b);

    // Apply companding to R, G, and B channels
    if normal_r > 0.0031308 {
        normal_r = ( 1.055 * normal_r.powf(micromath::F32::from(1.0 / 2.4)) ) - 0.055;
    } else {
        normal_r *= micromath::F32::from(12.92)
    }

    if normal_g > 0.0031308 {
        normal_g = ( 1.055 * normal_g.powf(micromath::F32::from(1.0 / 2.4)) ) - 0.055;
    } else {
        normal_g *= micromath::F32::from(12.92)
    }

    if normal_b > 0.0031308 {
        normal_b = ( 1.055 * normal_b.powf(micromath::F32::from(1.0 / 2.4)) ) - 0.055;
    } else {
        normal_b *= micromath::F32::from(12.92)
    }

    let result_r = ( normal_r * 255.0 ).round().0 as u8;
    let result_g = ( normal_g * 255.0 ).round().0 as u8;
    let result_b = ( normal_b * 255.0 ).round().0 as u8;
    Rgb565::new(result_r, result_g, result_b)
}


