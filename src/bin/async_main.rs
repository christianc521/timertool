#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_alloc::HeapStats;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, gpio::{Input, InputConfig, Pull}, interrupt::software::SoftwareInterruptControl, peripherals::PSRAM, psram::{self, psram_raw_parts}, system::Stack, timer::timg::TimerGroup};
use static_cell::StaticCell;
use timetool_v2::{button::Button, clock::{DoubleTimerSession, SessionNotifier, SessionState}, tft::{SpiPins, TFT}};
use timetool_v2::constants::PSRAM_ALLOCATOR;
esp_bootloader_esp_idf::esp_app_desc!();


#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::println!("Init!");

    static APP_CORE_STACK: StaticCell<Stack<8192>> = StaticCell::new();
    let app_core_stack = APP_CORE_STACK.init(Stack::new());
    // init_psram_heap(&peripherals.PSRAM);

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let stats: HeapStats = PSRAM_ALLOCATOR.stats();
    esp_println::println!("{}", stats);

    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let cs = peripherals.GPIO10;   
    let dc = peripherals.GPIO9;   
    let mosi = peripherals.GPIO11; 
    let miso = peripherals.GPIO13; 
    let sclk = peripherals.GPIO12;
    let rst = peripherals.GPIO4;
    let dma = peripherals.DMA_CH0;
    let spi_pins = SpiPins {
        dma,
        spi2: peripherals.SPI2,
        sclk,
        miso,
        mosi,
        cs,
        rst,
        dc
    };

    let tft = TFT::new(spi_pins);

     let config = InputConfig::default().with_pull(Pull::Down);
     let input = Input::new(peripherals.GPIO16, config);
     let mut button = Button::new(input);

     let mut state = SessionState::default();
     static SESSION_NOTIFIER: SessionNotifier = DoubleTimerSession::notifier();
     let mut session = DoubleTimerSession::new(tft, spawner, &SESSION_NOTIFIER).unwrap();

    // // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        esp_println::println!("im in da embussy :3");
        state = state.execute(&mut session, &mut button).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}

fn init_psram_heap(psram: &PSRAM<'_>) {
    unsafe {
        let (start, size) = psram_raw_parts(psram);
        PSRAM_ALLOCATOR.add_region(esp_alloc::HeapRegion::new(
                start,
                size,
                esp_alloc::MemoryCapability::External.into()));
    }
}
