use crate::{animations::Animation, scenes_util::SceneData};

use crate::clock::SessionState;

pub struct Packet(pub Payload);

#[derive(Debug, Clone, Copy)]
pub enum Payload {
    Time([u8; 20], SessionState),
    Animate(Animation),
    NewScene(SceneData),
    Menu,
    Empty
}

impl Default for Packet {
    fn default() -> Self {
        Packet(Payload::Menu)
    }
}

impl Packet {
    pub fn from_time(time: [u8; 20], timer: SessionState) -> Self {
        let payload = Payload::Time(time, timer);
        Packet(payload)
    }

    pub fn menu() -> Self {
        Packet(Payload::Menu)
    }
}
