use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker, Timer};

use crate::tft::TFT;
use crate::payloads::Packet;
use crate::constants::FRAME_RATE;

pub type TFTNotifier = Signal<CriticalSectionRawMutex, Packet>;
pub struct TFTRender<'a>(&'a TFTNotifier);

impl TFTRender<'_> {
    #[must_use]
    pub const fn notifier() -> TFTNotifier {
        Signal::new()
    }

    pub fn new(
        tft: TFT<'static>,
        notifier: &'static TFTNotifier,
        spawner: Spawner
        ) -> Result<Self, SpawnError> {
        spawner.spawn(render_loop(tft, notifier))?;
        Ok(Self(notifier))
    }

    // called by Session
    pub fn render(&self, frame: Packet) {
       self.0.signal(frame); 
    }
}

#[embassy_executor::task]
async fn render_loop(
    tft: TFT<'static>,
    notifier: &'static TFTNotifier
) -> ! {
    // safely start state loop
    let err = inner_render_loop(tft, notifier).await;
}

// final step; draws to the display
async fn inner_render_loop(
    mut tft: TFT<'static>,
    notifier: &'static TFTNotifier
) -> ! {
    let packet = Packet::default();
    tft.handle_payload(&packet);
    let mut frame_ticker = Ticker::every(Duration::from_hz(FRAME_RATE));
    'outer: loop {

        // Hybrid Rendering System
        // 30 FPS while playing animations
        // Event-driven renders for state changes

        // handle any incoming event payloads first [high priority] 

        if !tft.playing_animation {
            // either wait for a new payload or sleep for 4 seconds
            let sleep1sec_or_signal = 
                select(
                    Timer::after_secs(1), 
                    notifier.wait()
                ).await;

            // if a new payload was recieved before the sleep, 
            // start loop with new payload
            if let Either::Second(notification) = sleep1sec_or_signal {
                tft.handle_payload(&notification);
                continue 'outer
            }
        } else {
            // either wait for a new payload or wait for the next draw frame
            let sleep30hz_or_signal = 
                select(
                    frame_ticker.next(), 
                    notifier.wait()
                ).await;

            // TODO: call tft render_next_frame on all animated elements
            tft.render_next_frame();

            // if a new payload was recieved before the next draw frame (30fps), 
            // start loop with new payload
            if let Either::Second(notification) = sleep30hz_or_signal {
                tft.handle_payload(&notification);
                continue 'outer
            }
        }

    }
}
