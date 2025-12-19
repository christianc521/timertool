use core::ops::AddAssign;
use embassy_time::{Duration, Instant};

struct SingleTime {
    last_update: Instant,
    seconds_running: Duration,
    is_running: bool
}

// Duration to be rendered on display
pub struct Time {
    offset: Duration,
    work_time: SingleTime,
    break_time: SingleTime,
    paused: bool
}

impl Default for Time {
    fn default() -> Self {
        let work_time = SingleTime {
            last_update: Instant::now(),
            seconds_running: Duration::from_secs(0),
            is_running: true
        };

        let break_time = SingleTime {
            last_update: Instant::now(),
            seconds_running: Duration::from_secs(0),
            is_running: false
        };

        Self {
            offset: Duration::from_millis(0),
            work_time,
            break_time,
            paused: false
        }
    }
}

impl Time {
    #[inline]
    pub fn now(&self) -> Duration {
        let ms = Instant::now().as_millis() + self.offset.as_millis();
        Duration::from_millis(ms)
    }

    // NOTE: This should take another argument of type SessionState.
    //       Update individual Self Duration fields based on this.
    //       Paused shouldn't increment?
    #[inline]
    pub fn sleep_for_work(&mut self) -> ([u8; 20], Duration) {
        let now = self.now();
        let sleep_duration = Self::until_next(now, Duration::from_secs(1));

        self.break_time.is_running = false;
        self.paused = false;
        if !self.work_time.is_running {
            self.work_time.last_update = Instant::now();
            self.work_time.is_running = true;
        }
        let elapsed = Instant::now() - self.work_time.last_update;
        self.work_time.seconds_running += elapsed;
        self.work_time.last_update = Instant::now();

        let seconds_now = self.work_time.seconds_running.as_secs();
        let hours = seconds_now / 3600;
        let minutes = (seconds_now % 3600) / 60;
        let seconds = seconds_now % 60;

        let time_arr = format_time(hours, minutes, seconds);
        ( time_arr, sleep_duration )
    }

    #[inline]
    pub fn sleep_for_break(&mut self) -> ([u8; 20], Duration) {
        let now = self.now();
        let sleep_duration = Self::until_next(now, Duration::from_secs(1));

        self.work_time.is_running = false;
        self.paused = false;
        if !self.break_time.is_running {
            self.break_time.last_update = Instant::now();
            self.break_time.is_running = true;
        }
        let elapsed = Instant::now() - self.break_time.last_update;
        self.break_time.seconds_running += elapsed;
        self.break_time.last_update = Instant::now();

        let seconds_now = self.break_time.seconds_running.as_secs();
        let hours = seconds_now / 3600;
        let minutes = (seconds_now % 3600) / 60;
        let seconds = seconds_now % 60;

        let time_arr = format_time(hours, minutes, seconds);
        ( time_arr, sleep_duration )
    }

    #[inline]
    pub fn sleep_for_pause(&mut self) -> ([u8; 20], Duration) {
        self.work_time.is_running = false;
        self.break_time.is_running = false;
        self.paused = true;
        ([0; 20], Duration::from_secs(1))
    }

    #[inline]
    pub const fn until_next(now: Duration, next: Duration) -> Duration {
        let next_ticks = next.as_ticks();
        Duration::from_ticks(next_ticks - now.as_ticks() % next_ticks)
    }
}

impl AddAssign<Duration> for Time {
    fn add_assign(&mut self, rhs: Duration) {
        let ms = self.offset.as_millis() + rhs.as_millis();
        self.offset = Duration::from_millis(ms)
    }
}

fn format_time(hours: u64, mins: u64, seconds: u64) -> [u8; 20] {
    let mut buffer = [b' '; 20]; // Initialize with spaces
    
    // Convert numbers to digits
    let h1 = (hours / 10) as u8 + b'0';
    let h2 = (hours % 10) as u8 + b'0';
    let m1 = (mins / 10) as u8 + b'0';
    let m2 = (mins % 10) as u8 + b'0';
    let s1 = (seconds / 10) as u8 + b'0';
    let s2 = (seconds % 10) as u8 + b'0';
    
    // Format: "HH:MM:SS.hh"
    buffer[0] = h1;
    buffer[1] = h2;
    buffer[2] = b':';
    buffer[3] = m1;
    buffer[4] = m2;
    buffer[5] = b':';
    buffer[6] = s1;
    buffer[7] = s2;
    buffer
}
