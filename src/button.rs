use embassy_futures::select::{select, Either};
use esp_hal::gpio::Input;
use embassy_time::{Duration, Timer};

pub struct Button<'a>(Input<'a>);

const DEBOUNCE_DELAY: Duration = Duration::from_millis(50);
const LONG_PRESS: Duration = Duration::from_millis(1000);


impl<'a> Button<'a> {
    pub const fn new(button: Input<'a>) -> Self {
        Self(button)
    }

    #[inline]
    async fn wait_for_button_up(&mut self) -> &mut Self {
        self.0.wait_for_low().await;
        esp_println::println!("waited for low");
        self
    }

    #[inline]
    async fn wait_for_button_down(&mut self) -> &mut Self {
        self.0.wait_for_high().await;
        self
    }

    pub async fn press_duration(&mut self) -> PressDuration {
        self.wait_for_button_up().await;
        Timer::after(DEBOUNCE_DELAY).await;
        self.wait_for_button_down().await;
        Timer::after(DEBOUNCE_DELAY).await;
        let press_duration = 
            match select(self.wait_for_button_up(), Timer::after(LONG_PRESS)).await {
                Either::First(_) => { 
                    esp_println::println!("Short Press!");
                    PressDuration::Short
                },
                Either::Second(()) => { 
                    esp_println::println!("Long Press!");
                    PressDuration::Long
                }
            };
        press_duration
    }

    #[inline]
    pub async fn wait_for_press(&mut self) -> &mut Self {
        self.0.wait_for_rising_edge().await;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PressDuration {
    Short,
    Long
}
