use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Ticker, Timer};
use embassy_sync::channel::Channel;
use crate::button::{Button, PressDuration};
use crate::payloads::Packet;
use crate::render_display::{ TFTNotifier, TFTRender };
use crate::tft::TFT;
use crate::time_util::Time;

/*
 * Represents a single Ticker that increments 'run_duration' every tenth of a second
 */
pub struct SingleClock {
    run_duration: Duration,
}

impl Default for SingleClock {
    fn default() -> Self {
        SingleClock { run_duration: Duration::from_ticks(0) }
    }
}

impl SingleClock {
    pub fn new() -> Self {
        SingleClock { 
            run_duration: Duration::from_ticks(0), 
        }
    }

    pub async fn run_clock(&mut self) {
        let mut ticker = Ticker::every(Duration::from_millis(100));
        loop {
            self.run_duration += Duration::from_millis(100);
            ticker.next().await;
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SessionState {
    #[default]
    Working,
    Break,
    Paused
}

impl SessionState {
    pub async fn execute(
        self, 
        session: &mut DoubleTimerSession<'_>, 
        button: &mut Button<'_>) -> Self 
    {
        match self {
            SessionState::Working => self.execute_working(session, button).await,
            SessionState::Break => self.execute_break(session, button).await,
            SessionState::Paused => self.execute_paused(session, button).await,
        }
    }

    async fn execute_working(self, session: &mut DoubleTimerSession<'_>, button: &mut Button<'_>) -> Self {
        session.set_state(self).await;
        match button.press_duration().await {
            PressDuration::Short => {
                esp_println::println!("working -> break (short)");
                Self::Break
            }
            PressDuration::Long => {
                esp_println::println!("working -> paused (long)");
                Self::Paused
            }
        }
    }

    async fn execute_break(self, session: &mut DoubleTimerSession<'_>, button: &mut Button<'_>) -> Self {
        session.set_state(self).await;
        match button.press_duration().await {
            PressDuration::Short => {
                esp_println::println!("break -> working (short)");
                Self::Working
            }
            PressDuration::Long => {
                esp_println::println!("working -> paused (long)");
                Self::Paused
            }
        }
    }

    async fn execute_paused(self, session: &mut DoubleTimerSession<'_>, button: &mut Button<'_>) -> Self {
        session.set_state(self).await;
        match button.press_duration().await {
            PressDuration::Short => {
                esp_println::println!("pause -> working (short)");
                Self::Working
            }
            PressDuration::Long => {
                esp_println::println!("pause -> break (long)");
                Self::Break
            }
        }
    }

    pub(crate) fn render(self, time: &mut Time) -> (Packet, Duration) {
        match self {
            Self::Working => Self::render_working(time),
            Self::Break => Self::render_break(time),
            Self::Paused => Self::render_paused(time)
        }
    }

    fn render_working(time: &mut Time) -> (Packet, Duration) {
        let (display_time, sleep_dur) = time.sleep_for_work();
        let panel = Packet::from_time(display_time, SessionState::Working);
        (panel, sleep_dur)
    }

    fn render_break(time: &mut Time) -> (Packet, Duration) {
        let (display_time, sleep_dur) = time.sleep_for_break();
        let panel = Packet::from_time(display_time, SessionState::Break);
        (panel, sleep_dur)
    }

    fn render_paused(time: &mut Time) -> (Packet, Duration) {
        let (display_time, sleep_dur) = time.sleep_for_pause();
        let panel = Packet::from_time(display_time, SessionState::Paused);
        (panel, sleep_dur)
    }

}

pub enum SessionNotice {
    SetState(SessionState),
    AdjustTimer(Duration)
}

impl SessionNotice {
    pub(crate) fn apply(self, time: &mut Time, state: &mut SessionState) {
        match self {
            Self::AdjustTimer(delta) => {
                *time += delta
            }
            Self::SetState(new_state) => {
                *state = new_state
            }
        }
    }
}

pub type SessionNotifier = (SessionOuterNotifier, TFTNotifier);
pub type SessionOuterNotifier = Channel<CriticalSectionRawMutex, SessionNotice, 4>;

pub struct DoubleTimerSession<'spi>(&'spi SessionOuterNotifier);

impl<'spi> DoubleTimerSession<'spi> {
    pub fn new(
        tft: TFT<'static>,
        spawner: Spawner,
        notifier: &'static SessionNotifier,
    ) -> Result<Self, SpawnError> {
        let (outer_notifier, tft_notifier) = notifier;
        let tft = TFTRender::new(tft, tft_notifier, spawner)?;
        spawner.spawn(device_loop(outer_notifier, tft))?;
        Ok(Self(outer_notifier))
    }

    pub(crate) async fn set_state(&self, new_state: SessionState) {
        self.0.send(SessionNotice::SetState(new_state)).await;
    }

    #[must_use]
    pub const fn notifier() -> SessionNotifier {
        (Channel::new(), TFTRender::notifier())
    }

}

#[embassy_executor::task]
async fn device_loop(session_notifier: &'static SessionOuterNotifier, tft_renderer: TFTRender<'static>) -> ! {
    let mut time = Time::default();
    let mut session_state = SessionState::default();

    loop {
        let (panel, sleep_dur) = session_state.render(&mut time);
        tft_renderer.render(panel);
        if let Either::First(notification) = select(session_notifier.receive(), Timer::after(sleep_dur)).await
        {
            notification.apply(&mut time, &mut session_state);
        }
    }
}
