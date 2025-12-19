use core::{array::IntoIter, slice::Iter};

use allocator_api2::boxed::Box;
use embedded_graphics::prelude::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::Pixel;
use embedded_graphics_framebuf::{PixelIterator, backends::FrameBufferBackend};
use esp_alloc::ExternalMemory;
use esp_hal::system::Error;
use ili9341::DisplayError;
use crate::constants::PIXEL_COUNT;

pub struct BufferData(pub Box<[Rgb565; PIXEL_COUNT], ExternalMemory>);

impl FrameBufferBackend for BufferData {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Self::Color) {
        unsafe {
            *self.0.get_unchecked_mut(index) = color
        }

    }

    fn get(&self, index: usize) -> Self::Color {
        self.0[index]
    }

    fn nr_elements(&self) -> usize {
        self.0.len()
    }
}

impl OriginDimensions for BufferData {
    fn size(&self) -> embedded_graphics::prelude::Size {
        Size::new(320, 240)
    }
}

impl<'a> IntoIterator for &'a BufferData {
    type Item = Rgb565;
    type IntoIter = BufferDataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BufferDataIter {
            iter: self.0.iter()
        }
    }
}

pub struct BufferDataIter<'a> {
    iter: Iter<'a, Rgb565>
}

impl <'a> Iterator for BufferDataIter<'a> {
    type Item = Rgb565;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for BufferDataIter<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}



