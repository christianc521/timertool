use embedded_graphics::primitives::Rectangle;
use crate::scenes_util::{UIAction, UINode, SceneData};

#[derive(Debug, Clone, Copy)]
pub struct ClickableElement {
    pub position: Rectangle,
    pub value: u8,
    next_element: u8,
    prev_element: u8
}

impl UINode for ClickableElement {
    fn get_position(&self) -> &Rectangle {
        &self.position
    }

    fn handle_action(&mut self, scene: &mut SceneData, action: UIAction) {
       match action {
            UIAction::MoveBack => {
                scene.cursor_index = self.prev_element;
            }
            UIAction::MoveNext => {
                scene.cursor_index = self.next_element;
            }
            UIAction::Select => {
                self.value += 1;
            }
            UIAction::Back => {
                scene.cursor_index = self.prev_element;
            }
       }
    }
}
