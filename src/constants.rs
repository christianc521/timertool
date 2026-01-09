use embedded_graphics::{pixelcolor::Rgb565, prelude::Point};

use crate::{animations::{Animation, AnimationIterator, AnimationMetadata}, scenes_util::{Scene, SceneData, UIType}};

pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;

pub const FRAME_RATE: u64 = 15;
pub const PIXEL_COUNT: usize = 76800; 
pub const MAX_DIRTY_RECTS: usize = 4;
pub const MERGE_THRESHOLD: i32 = 16;
pub static PSRAM_ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub const DICE_ANIMATION: AnimationMetadata = AnimationMetadata::new(
    include_bytes!("./assets/dice_rgb565.bin"), 
    110, 
    75, 
    24);

pub const MIKU: AnimationMetadata = AnimationMetadata::new(
    include_bytes!("./assets/miku.bin"),
    150,
    20,
    10,
    );

pub const DICE_ITERATOR: AnimationIterator = AnimationIterator {
    frame_bytes: &DICE_ANIMATION,
    current_frame: 0,
    position: Point::new(20, 80),
    looping: true
};

pub const MIKU_ITERATOR: AnimationIterator = AnimationIterator {
    frame_bytes: &MIKU,
    current_frame: 0,
    position: Point::new(160, 80),
    looping: true
};

pub const MIKU_ITERATOR2: AnimationIterator = AnimationIterator {
    frame_bytes: &MIKU,
    current_frame: 0,
    position: Point::new(160, 110),
    looping: true
};

pub const MAX_ANIMATIONS: usize = 6;

pub const TEST_SCENE: SceneData = SceneData {
    scene: Scene::ConfigTaro,
    elements: [
        UIType::AnimatedSprite(Animation::Sprite(DICE_ITERATOR)),
        UIType::AnimatedSprite(Animation::Sprite(MIKU_ITERATOR)),
        UIType::AnimatedSprite(Animation::Sprite(MIKU_ITERATOR2)),
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

pub const MENU_HEADER_DATA: &[u8] = include_bytes!("./assets/Menu_Header.bmp");
pub const CLOCK_FACE_DATA: &[u8] = include_bytes!("./assets/Clock_Face.bmp");

pub const MAIN_MENU_SCENE: SceneData = SceneData {
    scene: Scene::MainMenu,
    elements: [
        UIType::Title,
        UIType::Empty,
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

pub const RGB_DEEP_PURPLE: Rgb565 = Rgb565::new(61, 56, 70);
