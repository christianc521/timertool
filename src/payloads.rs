use crate::{animations::Animation, scenes_util::SceneData};

use crate::clock::SessionState;

pub struct Packet(pub Payload);

#[derive(Debug, Clone, Copy)]
pub enum Payload {
    Time([u8; 20], SessionState),
    Animate(Animation),
    NewScene(SceneData),
    Empty
}

impl Default for Packet {
    fn default() -> Self {
        let empty_time = Payload::Time([0; 20], SessionState::Break);
        Packet(empty_time)
    }
}

impl Packet {
    pub fn from_time(time: [u8; 20], timer: SessionState) -> Self {
        let payload = Payload::Time(time, timer);
        Packet(payload)
    }
}
