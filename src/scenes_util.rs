use embedded_graphics::{Drawable, image::{Image, ImageRaw, ImageRawLE}, pixelcolor::Rgb565, prelude::{Dimensions, Point, RgbColor, Size}, primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment, StyledDimensions, StyledDrawable}, text::Text};
use embedded_graphics_framebuf::FrameBuf;
use embedded_ttf::FontTextStyleBuilder;
use rusttype::Font;
use crate::{animations::{Animation, AnimationState, FrameType}, clickable::ClickableElement, constants::{MAX_ANIMATIONS, MENU_HEADER_DATA, TEST_SCENE}, text_box::TextElement};

#[derive(Default, Debug, Clone, Copy)]
pub enum Scene {
    #[default]
    MainMenu,
    ConfigTaro,
    ConfigTaroPlus,
    ConfigCountingUp,
}

pub trait UINode {
    fn get_position(&self) -> &Rectangle;

    fn handle_action(&mut self, scene: &mut SceneData, action: UIAction);
}

#[derive(Debug, Clone, Copy)]
pub enum UIType<'a> {
    Menu(),
    Clickable(ClickableElement),
    Digits(DigitsElement),
    AnimatedSprite(Animation),
    TextBox(TextElement),
    Image(&'a [u8], u32),
    Title,
    Empty
}

impl Drawable for UIType <'_>{
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        match self {
            UIType::TextBox(text_element) => {
                let background_style = PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::new(
                            222, 
                            221, 
                            218))
                    .stroke_width(2)
                    .stroke_color(Rgb565::BLACK)
                    .build();
                text_element.position.draw_styled(&background_style, target)?;

                let top_bar = Rectangle::new(
                    text_element.position.top_left,
                    Size::new(
                        text_element.position.size.width, 
                        10)
                    );
                let style = PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::new(
                            192, 
                            191, 
                            188))
                    .stroke_width(2)
                    .stroke_color(Rgb565::BLACK)
                    .build();
                top_bar.draw_styled(&style, target)
            },
            UIType::Title => {
                const HEADER_WIDTH: u32 = 176;

                let raw_image: ImageRawLE<Rgb565> = ImageRaw::new(MENU_HEADER_DATA, HEADER_WIDTH);

                let image = Image::new(
                    &raw_image,
                    Point::new(138, 11)
                );

                image.draw(target)
            }
            _ => Ok(())
        }
    }
}

pub enum UIAction {
    Back,
    Select,
    MoveBack,
    MoveNext
}

#[derive(Default)]
pub struct SceneManager {
    pub current_scene: SceneData,
    pub animation_queue: AnimationState
}

impl SceneManager {
    pub fn initialize_scene(&mut self, new_scene: SceneData) {
        self.current_scene = new_scene;
        self.animation_queue = AnimationState::default();

        let mut animation_count = 0;
        self.current_scene.elements.iter().for_each(|element|
        {
            if let UIType::AnimatedSprite(element_data) = element {
                if animation_count < MAX_ANIMATIONS {
                    self.animation_queue.queue[animation_count] = *element_data;
                    animation_count += 1;
                }
            } 
        });
    }

    pub fn play_next(&mut self) -> [FrameType; 6] {
        let mut frames = [FrameType::Empty; 6];

        for ( index, animation ) in self.animation_queue
            .queue
            .iter_mut()
            .enumerate() {
            match animation {
                // If no animation queued here, do nothing
                Animation::Empty => {
                }
                _ => {
                    match animation.next_frame() {
                        Some(frame) => {
                            frames[index] = frame;
                        },
                        None => {
                            // Animation finished
                            *animation = Animation::Empty;
                            frames[index] = FrameType::Empty;
                        }
                    }
                }
            }
        }
        frames
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SceneData {
    pub scene: Scene,
    pub elements: [UIType; 10],
    pub cursor_index: u8,
}

impl Default for SceneData {
    fn default() -> Self {
        TEST_SCENE
    }
}


#[derive(Debug, Clone, Copy)]
pub struct DigitsElement {
    pub position: Rectangle,
    pub current_digit: u8,
    next_element: u8,
    prev_element: u8
}

impl UINode for DigitsElement 
{
   fn get_position(&self) -> &Rectangle {
        &self.position
   } 

   fn handle_action(&mut self, scene: &mut SceneData, action: UIAction) {
       match action {
            UIAction::MoveBack => {
                if self.current_digit == 0 {
                    self.current_digit = 9;
                }
                self.current_digit -= 1;
            }
            UIAction::MoveNext => {
                if self.current_digit == 9 {
                    self.current_digit = 0;
                }
                self.current_digit += 1;
            }
            UIAction::Select => {
                scene.cursor_index = self.next_element;
            }
            UIAction::Back => {
                scene.cursor_index = self.prev_element;
            }
       }
   }
}
