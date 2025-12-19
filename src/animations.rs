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

pub trait AnimationEvent {
    fn get_frame(&self) -> FrameType;
}

impl AnimationEvent for Animation {
    fn get_frame(&self) -> FrameType {
        match self {
            Self::Cursor(cursor_data) => FrameType::Empty,
            Self::Sprite(sprite_data) => sprite_data.get_frame_bytes(),
            Self::Empty => FrameType::Empty
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
    pub data: &'static [u8],
    pub width: u16,
    pub height: u16,
    pub frame_size: usize,
    pub frame_count: usize,
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
    pub position: Point
}

impl AnimationIterator {
    pub fn get_frame_bytes(&self) -> FrameType {
        let frame_byte_size = self.frame_bytes.frame_size * 2;
        let start_position = self.current_frame as usize * frame_byte_size;
        let end_position = start_position + frame_byte_size;

        let bytes = &self.frame_bytes.data[start_position..end_position];

        FrameType::Sprite(FrameData { 
            data: bytes,
            width: self.frame_bytes.width,
            height: self.frame_bytes.height,
            position: self.position
        })
    }
}

