use embedded_graphics::{prelude::Point, primitives::{line::Line, Rectangle}};
use embedded_graphics::prelude::*;
use embedded_graphics::geometry::AnchorPoint;
use crate::constants::MAX_ANIMATIONS;

#[derive(Debug, Copy, Clone)]
pub struct AnimationState {
    pub queue: [Animation; MAX_ANIMATIONS],
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState {
            queue: [Animation::Empty; MAX_ANIMATIONS]
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Animation {
    Cursor(CursorMove),
    Sprite(AnimationIterator),
    Empty
}

impl Animation {
    // Get next frame and advance animation state
    // Returns None when animation is complete
    pub fn next_frame(&mut self) -> Option<FrameType> {
        match self {
            Self::Cursor(cursor) => cursor.next_frame(),
            Self::Sprite(sprite) => sprite.next_frame(),
            Self::Empty => None,
        }
    }

    // Peek at current frame without advancing
    pub fn current_frame(&self) -> FrameType {
        match self {
            Self::Cursor(cursor) => cursor.current_frame(),
            Self::Sprite(sprite) => sprite.current_frame(),
            Self::Empty => FrameType::Empty,
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Cursor(cursor) => cursor.frame_index = 0,
            Self::Sprite(sprite) => sprite.current_frame = 0,
            Self::Empty => {}
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Cursor(cursor) => cursor.frame_index >= cursor.frame_count,
            Self::Sprite(sprite) => sprite.current_frame >= sprite.frame_bytes.frame_count,
            Self::Empty => true
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    Rectangle(Rectangle),
    Sprite(FrameData),
    Empty
}

#[derive(Debug, Clone, Copy)]
pub struct FrameData {
    pub data: &'static [u8], // Pixels in a single frame
    pub width: u16,
    pub height: u16,
    pub position: Point,
}

// Data to be fetched and stored on boot
#[derive(Debug)]
pub struct AnimationMetadata {
    // Raw RGB565 pixel data (little-endian, all frames concatenated)
    pub data: &'static [u8],
    pub width: u16,
    pub height: u16,
    // Size in BYTES per frame (width * height * 2 for RGB565)
    pub frame_size: usize,
    pub frame_count: usize,
}

impl AnimationMetadata {
    pub const fn new(
        data: &'static [u8],
        width: u16,
        height: u16,
        frame_count: usize,
        ) -> Self {
        Self {
            data,
            width,
            height,
            frame_size: (width as usize) * (height as usize) * 2,
            frame_count,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CursorMove {
    pub start: Point,
    pub end: Point,
    pub cursor_rect: Rectangle,
    pub path: Line,
    pub frame_count: usize,
    pub frame_index: usize
}

impl CursorMove {
    pub fn initialize(start_pos: Point, end_pos: Point) -> Self {
        let cursor = Rectangle::with_corners(Point::zero(), start_pos);
        let path = Line::new(start_pos, end_pos);
        let frame_count = path.points().count();

        Self {
            start: start_pos,
            end: end_pos,
            cursor_rect: cursor,
            path,
            frame_count,
            frame_index: 0
        }
    }

    pub fn next_frame(&mut self) -> Option<FrameType> { 
        if self.frame_index >= self.frame_count {
            return None;
        };

        let position = self.path.points().nth(self.frame_index)?;

        let x = position.x as u32;
        let y = position.y as u32;
        self.cursor_rect = self.cursor_rect
            .resized(
                Size::new(x, y), 
                AnchorPoint::TopLeft);

        self.frame_index += 1;
        Some(FrameType::Rectangle(self.cursor_rect))
    }

    pub fn current_frame(&self) -> FrameType {
        if self.frame_index >= self.frame_count {
            return FrameType::Empty;
        }
        FrameType::Rectangle(self.cursor_rect)
    }

    pub fn get_frame(&mut self) -> FrameType {
        let mut frame = FrameType::Empty;
        if self.frame_index >= self.frame_count {
            let position = self.path.points()
                .nth(self.frame_index as usize)
                .unwrap();

            let x = position.x as u32;
            let y = position.y as u32;
            self.cursor_rect = self.cursor_rect.resized(Size::new(x, y), AnchorPoint::TopLeft);
            
            self.frame_index += 1;
            frame = FrameType::Rectangle(self.cursor_rect.clone())
        }
        frame
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationIterator {
    pub frame_bytes: &'static AnimationMetadata,
    pub current_frame: usize,
    pub position: Point,
    pub looping: bool,
}

impl AnimationIterator {
    pub fn next_frame(&mut self) -> Option<FrameType> {
        if self.current_frame >= self.frame_bytes.frame_count {
            if self.looping {
                self.current_frame = 0;
            } else {
                return None;
            }
        }

        let frame = self.get_frame_at(self.current_frame);
        self.current_frame += 1;
        Some(frame)
    }

    pub fn current_frame(&self) -> FrameType {
        if self.current_frame >= self.frame_bytes.frame_count {
            return FrameType::Empty;
        }
        self.get_frame_at(self.current_frame)
    }

    fn get_frame_at(&self, frame_index: usize) -> FrameType {
        let start = frame_index * self.frame_bytes.frame_size;
        let end = start + self.frame_bytes.frame_size;

        if end > self.frame_bytes.data.len() {
            return FrameType::Empty;
        }

        let bytes = &self.frame_bytes.data[start..end];

        FrameType::Sprite(FrameData { 
            data: bytes,
            width: self.frame_bytes.width, 
            height: self.frame_bytes.height,
            position: self.position 
        })
    }
}


