use core::slice::Iter;

use allocator_api2::boxed::Box;
use embedded_graphics::prelude::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_framebuf::backends::FrameBufferBackend;
use esp_alloc::ExternalMemory;
use crate::constants::{ DISPLAY_HEIGHT, DISPLAY_WIDTH, MAX_DIRTY_RECTS, MERGE_THRESHOLD, PIXEL_COUNT };

pub struct BufferData { 
    pub buffer: Box<[Rgb565; PIXEL_COUNT], ExternalMemory>, 
    dirty_regions: [Option<Rectangle>; MAX_DIRTY_RECTS],
    dirty_count: usize,
    full_redraw_needed: bool
}

impl BufferData {
    pub fn new(buffer: Box<[Rgb565; PIXEL_COUNT], ExternalMemory>) -> Self {
        Self { 
            buffer, 
            dirty_regions: [None; MAX_DIRTY_RECTS],
            dirty_count: 0,
            full_redraw_needed: true 
        }
    }

    // Mark a region as needing redraw
    pub fn mark_dirty(&mut self, rect: Rectangle) {
        if self.full_redraw_needed {
            return; // Skip, already doing full redraw
        }

        // Try to merge area with an existing dirty region
        for region in self.dirty_regions.iter_mut().take(self.dirty_count) {
            if let Some(existing) = region {
                if let Some(merged) = Self::try_merge(*existing, rect) {
                    *existing = merged;
                    return;
                }
            }
        }

        // Add new dirty region
        if self.dirty_count < MAX_DIRTY_RECTS {
            self.dirty_regions[self.dirty_count] = Some(rect);
            self.dirty_count += 1;
        }
        else {
            self.full_redraw_needed = true;
        }
    }

    fn try_merge(a: Rectangle, b: Rectangle) -> Option<Rectangle> {
        let a_right = a.top_left.x + a.size.width as i32;
        let a_bottom = a.top_left.y + a.size.height as i32;
        let b_right = b.top_left.x + b.size.width as i32;
        let b_bottom = b.top_left.y + b.size.height as i32;

        // Check if rectangles overlap or are adjacent (within 16px)
        let h_gap = (a.top_left.x - b_right).max(b.top_left.x - a_right);
        let v_gap = (a.top_left.y - b_bottom).max(b.top_left.y - a_bottom);

        if h_gap <= MERGE_THRESHOLD && v_gap <= MERGE_THRESHOLD {
            let min_x = a.top_left.x.min(b.top_left.x);
            let min_y = a.top_left.y.min(b.top_left.y);
            let max_x = a_right.max(b_right);
            let max_y = a_bottom.max(b_bottom);
            Some(Rectangle::new(
                    Point::new(min_x, min_y), 
                    Size::new((max_x - min_x) as u32, (max_y - min_y) as u32),
            ))
        } else {
            None
        }
    }

    pub fn take_dirty_regions(&mut self) -> DirtyRegionIter {
        let iter = if self.full_redraw_needed {
            DirtyRegionIter::FullScreen
        } else {
            DirtyRegionIter::Regions { 
                regions: self.dirty_regions,
                index: 0,
                count: self.dirty_count 
            }
        };

        // Reset tracking
        self.dirty_regions = [None; MAX_DIRTY_RECTS];
        self.dirty_count = 0;
        self.full_redraw_needed = false;

        iter
    }

    // Force full redraw on next frame
    pub fn invalidate_all(&mut self) {
        self.full_redraw_needed = true;
    }

    // Get pixel index from coordinates
    #[inline(always)]
    const fn pixel_index(x: u32, y: u32) -> usize {
        (y * DISPLAY_WIDTH + x) as usize
    }

    // DMA Transfer helper
    // Get raw slice for a rectangular region
    // Returns row-by-row slices for the region
    pub fn get_region_rows(&self, rect: &Rectangle) -> RegionRowIter<'_> {
        RegionRowIter { 
            pixels: &self.buffer,
            rect: *rect,
            current_row: 0,
        }
    }
}

pub enum DirtyRegionIter {
    FullScreen,
    Regions {
        regions: [Option<Rectangle>; MAX_DIRTY_RECTS],
        index: usize,
        count: usize,
    },
}

impl Iterator for DirtyRegionIter {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DirtyRegionIter::FullScreen => {
                *self = DirtyRegionIter::Regions { 
                    regions: [None; MAX_DIRTY_RECTS],
                    index: 0, 
                    count: 0 
                };

                Some(Rectangle::new(
                        Point::zero(), 
                        Size::new(
                            DISPLAY_WIDTH,
                            DISPLAY_HEIGHT)
                ))
            },
            DirtyRegionIter::Regions { regions, index, count } => {
                while *index < *count {
                    let i = *index;
                    *index += 1;
                    if let Some(rect) = regions[i] {
                        return Some(rect);
                    }
                }
                None
            }
        }
    }
}

pub struct RegionRowIter<'a> {
    pixels: &'a [Rgb565; PIXEL_COUNT],
    rect: Rectangle,
    current_row: u32,
}

impl<'a> Iterator for RegionRowIter<'a> {
    type Item = &'a [Rgb565];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.rect.size.height {
            return None;
        }

        let y = self.rect.top_left.y as u32 + self.current_row;
        let x_start = self.rect.top_left.x as u32;
        let x_end = x_start + self.rect.size.width;

        let start_index = BufferData::pixel_index(x_start, y);
        let end_index = BufferData::pixel_index(x_end, y);

        self.current_row += 1;
        Some(&self.pixels[start_index..end_index])
    }
}


impl FrameBufferBackend for BufferData {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Self::Color) {
        // SAFETY: embedded-graphics-framebuf guarantees valid indices
        unsafe {
            *self.buffer.get_unchecked_mut(index) = color
        }

    }

    fn get(&self, index: usize) -> Self::Color {
        // SAFETY: embedded-graphics-framebuf guarantees valid indices
        self.buffer[index]
    }

    fn nr_elements(&self) -> usize {
        self.buffer.len()
    }
}

impl OriginDimensions for BufferData {
    fn size(&self) -> embedded_graphics::prelude::Size {
        Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }
}

impl<'a> IntoIterator for &'a BufferData {
    type Item = Rgb565;
    type IntoIter = BufferDataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BufferDataIter {
            iter: self.buffer.iter()
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



