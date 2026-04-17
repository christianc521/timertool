#![cfg_attr(not(feature = "simulator"), no_std)]
// #![cfg_attr(feature = "simulator", no_std)]

#[cfg(feature = "simulator")]
extern crate alloc;

pub mod tft;
pub mod payloads;
pub mod constants;
pub mod buffer_backend;
pub mod animations;
pub mod scenes_util;
pub mod clickable;
pub mod text_box;
pub mod home_ui;
pub mod display_driver;
pub mod color_mixing;

#[cfg(not(feature = "simulator"))]
pub mod clock;
#[cfg(not(feature = "simulator"))]
pub mod time_util;
#[cfg(not(feature = "simulator"))]
pub mod render_display;
#[cfg(not(feature = "simulator"))]
pub mod button;
#[cfg(not(feature = "simulator"))]
pub mod encoder;
