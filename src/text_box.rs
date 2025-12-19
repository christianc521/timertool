use embedded_graphics::primitives::Rectangle;

use crate::scenes_util::UINode;

#[derive(Debug, Clone, Copy)]
pub struct TextElement {
    pub position: Rectangle,
    pub text: &'static str
}

impl UINode for DigitsElement {
    fn get_position(&self) -> &Rectangle {
        &self.position
    }
    
    fn handle_action(&mut self, _scene: &mut crate::scenes_util::SceneData, _action: crate::scenes_util::UIAction) {
    }
}

pub struct DigitsElement {
    pub position: Rectangle,
    pub current_digit: u8,
    next_element: u8,
    prev_element: u8
}
