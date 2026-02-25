use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;
use crate::scenes_util::UIAction;

pub struct RotaryEncoder<'a> {
    pin_a: Input<'a>,
    pin_b: Input<'a>,
    last_a: bool,
    last_b: bool,
}

const DEBOUNCE_DELAY: Duration = Duration::from_micros(500);

impl<'a> RotaryEncoder<'a> {
    pub fn new(pin_a: Input<'a>, pin_b: Input<'a>) -> Self {
        let last_a = pin_a.is_high();
        let last_b = pin_b.is_high();
        
        Self {
            pin_a,
            pin_b,
            last_a,
            last_b,
        }
    }

    /// Wait for encoder rotation and return the corresponding UIAction
    pub async fn wait_for_rotation(&mut self) -> UIAction {
        loop {
            // Wait for any edge on pin A
            self.pin_a.wait_for_any_edge().await;
            Timer::after(DEBOUNCE_DELAY).await;
            
            let current_a = self.pin_a.is_high();
            let current_b = self.pin_b.is_high();
            
            // Only process if A actually changed (debounce check)
            if current_a != self.last_a {
                let direction = if current_a {
                    // A rising edge: check B to determine direction
                    if !current_b {
                        Some(RotationDirection::Clockwise)
                    } else {
                        Some(RotationDirection::CounterClockwise)
                    }
                } else {
                    // A falling edge: check B (inverted logic)
                    if current_b {
                        Some(RotationDirection::Clockwise)
                    } else {
                        Some(RotationDirection::CounterClockwise)
                    }
                };
                
                self.last_a = current_a;
                self.last_b = current_b;
                
                if let Some(dir) = direction {
                    return dir.into();
                }
            }
        }
    }

    /// Non-blocking check for rotation (returns None if no rotation detected)
    pub fn poll_rotation(&mut self) -> Option<UIAction> {
        let current_a = self.pin_a.is_high();
        let current_b = self.pin_b.is_high();
        
        if current_a != self.last_a {
            let direction = if current_a {
                if !current_b {
                    Some(RotationDirection::Clockwise)
                } else {
                    Some(RotationDirection::CounterClockwise)
                }
            } else {
                if current_b {
                    Some(RotationDirection::Clockwise)
                } else {
                    Some(RotationDirection::CounterClockwise)
                }
            };
            
            self.last_a = current_a;
            self.last_b = current_b;
            
            return direction.map(|d| d.into());
        }
        
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl From<RotationDirection> for UIAction {
    fn from(direction: RotationDirection) -> Self {
        match direction {
            RotationDirection::Clockwise => UIAction::MoveNext,
            RotationDirection::CounterClockwise => UIAction::MoveBack,
        }
    }
}
