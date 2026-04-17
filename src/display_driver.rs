use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub trait DisplayDriver: DrawTarget<Color = Rgb565> + OriginDimensions
    where
        Self::Error: core::fmt::Debug,
{
    fn set_pixel_region(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: impl IntoIterator<Item = Rgb565>,
        );
}

// ----------- Hardware Backend: mipidsi -----------------
// ILI9488 / ILI9341

#[cfg(feature = "ili9341")]
mod hardware_impl {
    use super::*;
    use crate::tft::{TFTDisplay, TFTSpiInterface};
    use esp_hal::gpio::Output;
    use mipidsi::{Display, models::ILI9488Rgb565};

    impl<'spi> DisplayDriver for Display<TFTSpiInterface<'spi>, ILI9488Rgb565, Output<'spi>> {
        fn set_pixel_region(
            &mut self,
            sx: u16,
            sy: u16,
            ex: u16,
            ey: u16,
            colors: impl IntoIterator<Item = Rgb565>,
            ) {
            self.set_pixels(sx, sy, ex, ey, colors).unwrap();
        }
    }
}

#[cfg(feature = "simulator")]
mod simulator_impl {
    use super::*;
    use embedded_graphics::primitives::Rectangle;
    use embedded_graphics_simulator::SimulatorDisplay;

    impl DisplayDriver for SimulatorDisplay<Rgb565> {
        fn set_pixel_region(
            &mut self,
            sx: u16,
            sy: u16,
            ex: u16,
            ey: u16,
            colors: impl IntoIterator<Item = Rgb565>,
            ) {
            let w = (ex - sx + 1) as u32;
            let h = (ey - sy + 1) as u32;
            let area = Rectangle::new(
                Point::new(sx as i32, sy as i32), 
                Size::new(w, h)
            );
            self.fill_contiguous(&area, colors).unwrap();
        }
    }
}

