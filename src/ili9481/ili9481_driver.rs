#![no_std]

//! ILI9481 TFT Display Driver
//!
//! A `no_std` driver for ILI9481-based 320x480 TFT displays over SPI,
//! compatible with `embedded-graphics` and `display-interface`.
//!
//! Over SPI, the ILI9481 uses 18-bit color (3 bytes per pixel), so this
//! driver converts Rgb565 pixels to 18-bit RGB666 for transmission.

use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{ Rgb565, raw::RawU16 },
    primitives::Rectangle,
    Pixel,
};
use embedded_hal_async::delay::DelayNs;
use embedded_hal::digital::OutputPin;

// ──────────────────────────────────────────────
// ILI9481 Command Constants
// ──────────────────────────────────────────────

const CMD_NOP: u8 = 0x00;
const CMD_SWRST: u8 = 0x01;
const CMD_SLPIN: u8 = 0x10;
const CMD_SLPOUT: u8 = 0x11;
const CMD_INVOFF: u8 = 0x20;
const CMD_INVON: u8 = 0x21;
const CMD_DISPOFF: u8 = 0x28;
const CMD_DISPON: u8 = 0x29;
const CMD_CASET: u8 = 0x2A;
const CMD_PASET: u8 = 0x2B;
const CMD_RAMWR: u8 = 0x2C;
const CMD_RAMRD: u8 = 0x2E;
const CMD_MADCTL: u8 = 0x36;
const CMD_COLMOD: u8 = 0x3A;

// MADCTL bit flags
const MADCTL_MY: u8 = 0x80;
const MADCTL_MX: u8 = 0x40;
const MADCTL_MV: u8 = 0x20;
const MADCTL_ML: u8 = 0x10;
const MADCTL_BGR: u8 = 0x08;
const MADCTL_MH: u8 = 0x04;
const MADCTL_SS: u8 = 0x02;
const MADCTL_GS: u8 = 0x01;

/// Display width in portrait mode
const WIDTH: u16 = 320;
/// Display height in portrait mode
const HEIGHT: u16 = 480;

// ──────────────────────────────────────────────
// Public Types
// ──────────────────────────────────────────────

/// Display orientation
#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Portrait,
    Landscape,
    PortraitFlipped,
    LandscapeFlipped,
}

impl Orientation {
    /// Returns the MADCTL register value for this orientation.
    /// Base flags: BGR (0x08) | SS (0x02) = 0x0A, matching the ILI9481 init sequence.
    fn madctl(self) -> u8 {
        const BASE: u8 = MADCTL_BGR | MADCTL_SS; // 0x0A
        match self {
            Orientation::Portrait => BASE,
            Orientation::Landscape => BASE | MADCTL_MV | MADCTL_MX,
            Orientation::PortraitFlipped => BASE | MADCTL_MY | MADCTL_MX,
            Orientation::LandscapeFlipped => BASE | MADCTL_MV | MADCTL_MY,
        }
    }

    /// Returns (width, height) for the active orientation
    fn dimensions(self) -> (u16, u16) {
        match self {
            Orientation::Portrait | Orientation::PortraitFlipped => (WIDTH, HEIGHT),
            Orientation::Landscape | Orientation::LandscapeFlipped => (HEIGHT, WIDTH),
        }
    }
}

/// Marker type selecting Rgb565 colour mode.
/// The driver accepts `Rgb565` pixels and internally converts to the
/// 18-bit (3-byte) format required by the ILI9481 SPI interface.
pub struct Rgb565Mode;

/// ILI9481 init variant, corresponding to different panel types
/// found in TFT_eSPI ILI9481_Init.h
#[derive(Copy, Clone, Debug, Default)]
pub enum InitVariant {
    /// Original default init sequence
    #[default]
    Default,
    /// CPT29 panel
    Cpt29,
    /// PVI35 panel
    Pvi35,
    /// AUO317 panel
    Auo317,
}

/// Driver error type
#[derive(Debug)]
pub enum Error<BusError, PinError> {
    Bus(BusError),
    Pin(PinError),
}

impl<BE, PE> From<BE> for Error<BE, PE> {
    fn from(e: BE) -> Self {
        Error::Bus(e)
    }
}

// ──────────────────────────────────────────────
// Main Driver
// ──────────────────────────────────────────────

pub struct Ili9481<DI, RST, MODE> {
    interface: DI,
    rst: RST,
    orientation: Orientation,
    pub width: u16,
    pub height: u16,
    _mode: MODE,
}

impl<DI, RST> Ili9481<DI, RST, Rgb565Mode>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    /// Create a new ILI9481 driver instance with the default init variant.
    pub fn new(
        interface: DI,
        rst: RST,
        delay: &mut impl DelayNs,
        orientation: Orientation,
        _mode: Rgb565Mode,
    ) -> Result<Self, Error<DI, RST>> {
        Self::new_with_variant(interface, rst, delay, orientation, _mode, InitVariant::Default)
    }

    /// Create a new driver instance, choosing a specific panel init variant.
    pub fn new_with_variant(
        interface: DI,
        mut rst: RST,
        delay: &mut impl DelayNs,
        orientation: Orientation,
        _mode: Rgb565Mode,
        variant: InitVariant,
    ) -> Result<Self, Error<DI, RST>> {
        // Hardware reset
        rst.set_low().map_err(Error::Pin);
        delay.delay_ms(10);
        rst.set_high().map_err(Error::Pin);
        delay.delay_ms(120);

        let (width, height) = orientation.dimensions();
        let mut driver = Ili9481 {
            interface,
            rst,
            orientation,
            width,
            height,
            _mode,
        };

        driver.init_sequence(delay, variant)?;
        driver.set_orientation(orientation)?;

        Ok(driver)
    }

    // ── Initialisation ──────────────────────────

    fn init_sequence(
        &mut self,
        delay: &mut impl DelayNs,
        variant: InitVariant,
    ) -> Result<(), Error<DI, RST>> {
        // Sleep out
        self.write_command(CMD_SLPOUT)?;
        delay.delay_ms(20);

        match variant {
            InitVariant::Default => self.init_default()?,
            InitVariant::Cpt29 => self.init_cpt29()?,
            InitVariant::Pvi35 => self.init_pvi35()?,
            InitVariant::Auo317 => self.init_auo317()?,
        }

        // Pixel format: 18-bit for SPI
        self.write_command(CMD_COLMOD)?;
        self.write_data(&[0x66])?;

        // Inversion on (required for SPI)
        self.write_command(CMD_INVON)?;

        // Column address set: 0x0000..0x013F (0..319)
        self.write_command(CMD_CASET)?;
        self.write_data(&[0x00, 0x00, 0x01, 0x3F])?;

        // Page address set: 0x0000..0x01DF (0..479)
        self.write_command(CMD_PASET)?;
        self.write_data(&[0x00, 0x00, 0x01, 0xDF])?;

        delay.delay_ms(120);

        // Display on
        self.write_command(CMD_DISPON)?;
        delay.delay_ms(25);

        Ok(())
    }

    fn init_default(&mut self) -> Result<(), Error<DI, RST>> {
        // Power Setting
        self.write_command(0xD0)?;
        self.write_data(&[0x07, 0x42, 0x18])?;

        // VCOM Control
        self.write_command(0xD1)?;
        self.write_data(&[0x00, 0x07, 0x10])?;

        // Power Setting for Normal Mode
        self.write_command(0xD2)?;
        self.write_data(&[0x01, 0x02])?;

        // Panel Driving Setting
        self.write_command(0xC0)?;
        self.write_data(&[0x10, 0x3B, 0x00, 0x02, 0x11])?;

        // Frame Rate
        self.write_command(0xC5)?;
        self.write_data(&[0x03])?;

        // Gamma Setting
        self.write_command(0xC8)?;
        self.write_data(&[
            0x00, 0x32, 0x36, 0x45, 0x06, 0x16,
            0x37, 0x75, 0x77, 0x54, 0x0C, 0x00,
        ])?;

        Ok(())
    }

    fn init_cpt29(&mut self) -> Result<(), Error<DI, RST>> {
        self.write_command(0xD0)?;
        self.write_data(&[0x07, 0x41, 0x1D])?;

        self.write_command(0xD1)?;
        self.write_data(&[0x00, 0x2B, 0x1F])?;

        self.write_command(0xD2)?;
        self.write_data(&[0x01, 0x11])?;

        self.write_command(0xC0)?;
        self.write_data(&[0x10, 0x3B, 0x00, 0x02, 0x11])?;

        self.write_command(0xC5)?;
        self.write_data(&[0x03])?;

        self.write_command(0xC8)?;
        self.write_data(&[
            0x00, 0x14, 0x33, 0x10, 0x00, 0x16,
            0x44, 0x36, 0x77, 0x00, 0x0F, 0x00,
        ])?;

        self.write_command(0xB0)?;
        self.write_data(&[0x00])?;

        self.write_command(0xE4)?;
        self.write_data(&[0xA0])?;

        self.write_command(0xF0)?;
        self.write_data(&[0x01])?;

        self.write_command(0xF3)?;
        self.write_data(&[0x02, 0x1A])?;

        Ok(())
    }

    fn init_pvi35(&mut self) -> Result<(), Error<DI, RST>> {
        self.write_command(0xD0)?;
        self.write_data(&[0x07, 0x41, 0x1D])?;

        self.write_command(0xD1)?;
        self.write_data(&[0x00, 0x2B, 0x1F])?;

        self.write_command(0xD2)?;
        self.write_data(&[0x01, 0x11])?;

        self.write_command(0xC0)?;
        self.write_data(&[0x10, 0x3B, 0x00, 0x02, 0x11])?;

        self.write_command(0xC5)?;
        self.write_data(&[0x03])?;

        self.write_command(0xC8)?;
        self.write_data(&[
            0x00, 0x14, 0x33, 0x10, 0x00, 0x16,
            0x44, 0x36, 0x77, 0x00, 0x0F, 0x00,
        ])?;

        self.write_command(0xB0)?;
        self.write_data(&[0x00])?;

        self.write_command(0xE4)?;
        self.write_data(&[0xA0])?;

        self.write_command(0xF0)?;
        self.write_data(&[0x01])?;

        self.write_command(0xF3)?;
        self.write_data(&[0x40, 0x0A])?;

        Ok(())
    }

    fn init_auo317(&mut self) -> Result<(), Error<DI, RST>> {
        self.write_command(0xD0)?;
        self.write_data(&[0x07, 0x40, 0x1D])?;

        self.write_command(0xD1)?;
        self.write_data(&[0x00, 0x18, 0x13])?;

        self.write_command(0xD2)?;
        self.write_data(&[0x01, 0x11])?;

        self.write_command(0xC0)?;
        self.write_data(&[0x10, 0x3B, 0x00, 0x02, 0x11])?;

        self.write_command(0xC5)?;
        self.write_data(&[0x03])?;

        self.write_command(0xC8)?;
        self.write_data(&[
            0x00, 0x44, 0x06, 0x44, 0x0A, 0x08,
            0x17, 0x33, 0x77, 0x44, 0x08, 0x0C,
        ])?;

        self.write_command(0xB0)?;
        self.write_data(&[0x00])?;

        self.write_command(0xE4)?;
        self.write_data(&[0xA0])?;

        self.write_command(0xF0)?;
        self.write_data(&[0x01])?;

        Ok(())
    }

    // ── Orientation ─────────────────────────────

    /// Change the display orientation at runtime.
    pub fn set_orientation(
        &mut self,
        orientation: Orientation,
    ) -> Result<(), Error<DI, RST>> {
        self.orientation = orientation;
        let (w, h) = orientation.dimensions();
        self.width = w;
        self.height = h;

        self.write_command(CMD_MADCTL)?;
        self.write_data(&[orientation.madctl()])?;
        Ok(())
    }

    // ── Window / Pixel Writes ───────────────────

    /// Set the active drawing window (column and page address).
    pub fn set_window(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), Error<DI, RST>> {
        self.write_command(CMD_CASET)?;
        self.write_data(&[
            (x0 >> 8) as u8,
            x0 as u8,
            (x1 >> 8) as u8,
            x1 as u8,
        ])?;

        self.write_command(CMD_PASET)?;
        self.write_data(&[
            (y0 >> 8) as u8,
            y0 as u8,
            (y1 >> 8) as u8,
            y1 as u8,
        ])?;

        self.write_command(CMD_RAMWR)?;
        Ok(())
    }

    /// Write a slice of `Rgb565` pixels into a rectangular region.
    ///
    /// Coordinates are inclusive: the region spans from `(x0, y0)` to `(x1, y1)`.
    /// Pixels are consumed left-to-right, top-to-bottom.
    pub fn draw_raw_slice(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        pixels: &[Rgb565],
    ) -> Result<(), Error<DI, RST>> {
        self.set_window(x0, y0, x1, y1)?;
        self.write_pixels(pixels)?;
        Ok(())
    }

    /// Convert Rgb565 pixels to 18-bit RGB666 and send them.
    ///
    /// Each Rgb565 (5-6-5 bits) is expanded to three bytes:
    ///   R: (r5 << 3) | (r5 >> 2)   → 8-bit, top 6 bits used by display
    ///   G: (g6 << 2) | (g6 >> 4)   → 8-bit
    ///   B: (b5 << 3) | (b5 >> 2)   → 8-bit
    fn write_pixels(&mut self, pixels: &[Rgb565]) -> Result<(), Error<DI, RST>> {
        // Process in chunks to keep stack usage bounded.
        // 170 pixels × 3 bytes = 510 bytes on the stack per chunk.
        const CHUNK_PIXELS: usize = 170;
        let mut buf = [0u8; CHUNK_PIXELS * 3];

        for chunk in pixels.chunks(CHUNK_PIXELS) {
            let bytes_len = chunk.len() * 3;
            for (i, px) in chunk.iter().enumerate() {
                let raw = RawU16::from(*px).into_inner();
                let r5 = ((raw >> 11) & 0x1F) as u8;
                let g6 = ((raw >> 5) & 0x3F) as u8;
                let b5 = (raw & 0x1F) as u8;

                buf[i * 3] = (r5 << 3) | (r5 >> 2);
                buf[i * 3 + 1] = (g6 << 2) | (g6 >> 4);
                buf[i * 3 + 2] = (b5 << 3) | (b5 >> 2);
            }
            self.write_data(&buf[..bytes_len])?;
        }
        Ok(())
    }

    // ── Low-level bus helpers ───────────────────

    fn write_command(&mut self, cmd: u8) -> Result<(), Error<DI, RST>> {
        self.interface.send_commands(DataFormat::U8(&[cmd]));
        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), Error<DI, RST>> {
        self.interface.send_data(DataFormat::U8(data));
        Ok(())
    }
}

