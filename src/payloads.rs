use crate::{animations::Animation, scenes_util::SceneData};

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SessionState {
    #[default]
    MainMenu,
    Working,
    Break,
    Paused
}

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
