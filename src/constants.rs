use embedded_graphics::{prelude::{Point, Size}, primitives::Rectangle};

use crate::{animations::{Animation, AnimationIterator, AnimationMetadata}, scenes_util::{Scene, SceneData, UIType}, text_box::TextElement};

pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;

pub const FRAME_RATE: u64 = 30;
pub const PIXEL_COUNT: usize = 76800; 
pub const MAX_DIRTY_RECTS: usize = 4;
pub const MERGE_THRESHOLD: i32 = 16;
pub static PSRAM_ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub const DICE_ANIMATION: AnimationMetadata = AnimationMetadata {
    data: include_bytes!("./assets/dice_rgb565.bin"),
    width: 137,
    height: 100,
    frame_size: 137 * 100,
    frame_count: 30
};

pub const DICE_ITERATOR: AnimationIterator = AnimationIterator {
    frame_bytes: &DICE_ANIMATION,
    current_frame: 0,
    position: Point::new(20, 20),
    looping: true
};

pub const MAX_ANIMATIONS: usize = 6;

pub const TEST_SCENE: SceneData = SceneData {
    scene: Scene::ConfigTaro,
    elements: [
        UIType::AnimatedSprite(Animation::Sprite(DICE_ITERATOR)),
        UIType::TextBox(TextElement{
            position: Rectangle::new(Point::new_equal(130), Size::new_equal(80)),
            text: "texter"
        }),
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
        UIType::Empty,
    ],
    cursor_index: 0
};
