use allocator_api2::boxed::Box;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_alloc::ExternalMemory;
use esp_hal::{Async, delay::Delay, dma::{DmaRxBuf, DmaTxBuf}, dma_buffers, gpio::{ Level, Output, OutputConfig }, peripherals::{ DMA_CH0, GPIO4, GPIO9, GPIO10, GPIO11, GPIO12, GPIO13, SPI2 }, spi::master::{Config, Spi, SpiDmaBus}, time::Rate};
use display_interface_spi::SPIInterface;
use ili9341::{DisplaySize240x320, Ili9341, Orientation};
use crate::{animations::{Animation, FrameData, FrameType}, buffer_backend::BufferData, clock::SessionState, constants::{MAX_ANIMATIONS, PIXEL_COUNT}, scenes_util::{SceneData, SceneManager}};

use embedded_graphics::{
    pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment, StyledDrawable}, text::{Baseline, Text}
};
use embedded_graphics_framebuf::FrameBuf;
use eg_seven_segment::SevenSegmentStyleBuilder;
extern crate allocator_api2;

use crate::payloads::{Packet, Payload};

pub type TFTSpiDevice<'spi> = 
    ExclusiveDevice<SpiDmaBus<'spi, Async>, Output<'spi>, NoDelay>;

pub type TFTSpiInterface<'spi> = 
    SPIInterface<
        ExclusiveDevice<SpiDmaBus<'spi, Async>, Output<'spi>, NoDelay>,
        Output<'spi>
    >;

pub struct SpiPins<'spi> {
        pub dma: DMA_CH0<'spi>,
        pub spi2: SPI2<'spi>,
        pub sclk: GPIO12<'spi>,
        pub miso: GPIO13<'spi>,
        pub mosi: GPIO11<'spi>,
        pub cs: GPIO10<'spi>,
        pub rst: GPIO4<'spi>,
        pub dc: GPIO9<'spi>,
}

pub struct TFT<'spi>
{
    pub display: Ili9341<TFTSpiInterface<'spi>, Output<'spi>>,
    pub playing_animation: bool,
    frame_buffer: FrameBuf<Rgb565, BufferData>,
    scene_manager: SceneManager
}

impl<'spi> TFT<'spi> {
    pub fn new(
        spi_pins: SpiPins<'spi>,
        ) -> Self {
        let rst_output = Output::new(spi_pins.rst, Level::Low, OutputConfig::default());
        let dc_output = Output::new(spi_pins.dc, Level::Low, OutputConfig::default());

        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
        let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
        let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

        let spi = Spi::new(
            spi_pins.spi2, 
            Config::default()
                .with_frequency(Rate::from_mhz(60))
                .with_mode(esp_hal::spi::Mode::_0))
            .unwrap()
            .with_sck(spi_pins.sclk)
            .with_mosi(spi_pins.mosi)
            .with_miso(spi_pins.miso)
            .with_dma(spi_pins.dma)
            .with_buffers(dma_rx_buf, dma_tx_buf)
            .into_async();

        let cs_output = Output::new(spi_pins.cs, Level::Low, OutputConfig::default());
        let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();
        let interface = SPIInterface::new(spi_device, dc_output);

        // Create the buffer backend in PSRAM
        let boxed_buffer_data: Box<[Rgb565; PIXEL_COUNT], ExternalMemory> = Box::new_in([Rgb565::BLACK; PIXEL_COUNT], ExternalMemory);

        // FrameBuf implementation for PSRAM data
        let boxed_buffer_data = BufferData::new(boxed_buffer_data);
        let frame_buffer = FrameBuf::new(boxed_buffer_data, 320, 240);


        let display = Ili9341::new(
            interface, 
            rst_output, 
            &mut Delay::new(),
            Orientation::Landscape, 
            DisplaySize240x320
        ).unwrap();

        esp_println::println!("Initialized Display!");

        let mut tft = TFT { 
            display,
            playing_animation: true,
            frame_buffer,
            scene_manager: SceneManager::default()
        };
        tft.initialize_scene();
        tft
    }

    pub fn initialize_scene(&mut self) {
        self.frame_buffer.clear(Rgb565::BLACK).unwrap();
        self.scene_manager.initialize_scene(SceneData::default());

        for element in self.scene_manager.current_scene.elements {
            element.draw(&mut self.frame_buffer).unwrap();
        }

        self.display.fill_contiguous(&self.display.bounding_box(), &self.frame_buffer.data).unwrap();

        let _ = self.frame_buffer.data.take_dirty_regions();
    }

    pub fn handle_payload(&mut self, packet: &Packet) {
        let payload = packet.0;

        match payload {
            Payload::Time(bytes, state) => {
                let message = str::from_utf8(&bytes).unwrap_or("error");

                let ( color, point ) = match state {
                    SessionState::Working => ( Rgb565::new(123, 191, 255), Point::new(10, 20) ),
                    SessionState::Break => ( Rgb565::new(255, 148, 150), Point::new(10, 160) ),
                    _ => ( Rgb565::WHITE, Point::new(10, 95) )
                };
                self.render_segmented(color, point, message);
                // self.render_divider(state);
            },
            Payload::Animate(animation) => {
                // Only add the animation to the queue if there's space
                if let Some(index) = self.scene_manager
                    .animation_queue
                    .queue
                    .iter()
                    .position(|a| matches!(a, Animation::Empty)) {

                        self.scene_manager
                            .animation_queue
                            .queue[index] = animation;

                        self.playing_animation = true;

                        // Start rendering animation
                        self.render_next_frame();
                }
            }
            Payload::NewScene(new_scene) => {
                self.scene_manager.initialize_scene(new_scene);
            }
            _ => (),
        };
    }

    pub fn flush_dirty_regions(&mut self) {
        let dirty_regions: heapless::Vec<Rectangle, 8> = self
            .frame_buffer
            .data
            .take_dirty_regions()
            .collect();

        for region in dirty_regions {
            self.transfer_region(&region);
        }
    }

    fn transfer_region(&mut self, rect: &Rectangle) {
        let x0 = rect.top_left.x as u16;
        let y0 = rect.top_left.y as u16;
        let x1 = x0 + rect.size.width as u16 - 1;
        let y1 = y0 + rect.size.height as u16 - 1;

        // Set window once for the entire region
        // Row-by-row transfer:
        for (row_index, row_pixels) in self
            .frame_buffer
            .data
            .get_region_rows(rect)
            .enumerate() {
                let row_y = y0 + row_index as u16;

                // Convert &[Rgb565] to &[u16] for raw transfer
                let raw_pixels = unsafe {
                    core::slice::from_raw_parts(
                        row_pixels.as_ptr() as *const u16,
                        row_pixels.len(),
                    )
                };

                self.display
                    .draw_raw_slice(x0, row_y, x1, row_y, raw_pixels)
                    .unwrap();
            }
    }

    pub fn draw_and_mark_dirty<D: Drawable<Color = Rgb565>>(
        &mut self,
        drawable: &D,
    ) -> Result<D::Output, core::convert::Infallible> 
    where 
        D: DrawTarget<Color = Rgb565, Error = core::convert::Infallible>
    {
        // Get bounding box before drawing
        let bounds = drawable.bounding_box();

        let result = drawable.draw(&mut self.frame_buffer)?;

        self.frame_buffer.data.mark_dirty(bounds);

        Ok(result)
    }


    pub fn render_next_frame(&mut self) {

        // Grab array of frames to be rendered
        let frame_queue = self.scene_manager.play_next();

        // Empties flag; 
        // if equal to SceneManager animation_queue[] capacity,
        // all animations have been exhausted
        // set tft playing_animation to false
        let mut empty_count: usize = 0;
        for frame in frame_queue {
            match frame {
                FrameType::Rectangle(rect) => { 
                    // Only flush dirty regions
                    self.flush_dirty_regions();
                    self.animate_cursor(rect);
                    self.frame_buffer.data.mark_dirty(rect);
                },
                FrameType::Sprite(frame_data) => { 
                    self.render_frame(frame_data);

                    let sprite_rect = Rectangle::new(
                        frame_data.position,
                        Size::new(
                            frame_data.width as u32,
                            frame_data.height as u32
                            )
                        );
                    // self.frame_buffer.data.mark_dirty(sprite_rect);

                },
                FrameType::Empty => empty_count += 1,
            }
        }

        // Turn off 30 fps render flag if no more frames in the queue
        if empty_count == MAX_ANIMATIONS { self.playing_animation = false };
    }

    fn render_frame(&mut self, frame_data: FrameData) {
        let x0 = frame_data.position.x as u16;
        let y0 = frame_data.position.y as u16;
        let x1 = x0 + frame_data.width - 1;
        let y1 = y0 + frame_data.height - 1;

        let pixel_count = frame_data.width * frame_data.height;
        let bytes = unsafe {
            core::slice::from_raw_parts(
                frame_data.data.as_ptr() as *const u16,
                pixel_count as usize
                )
        };

        let _ = &self.display.draw_raw_slice(x0, y0, x1, y1, bytes).unwrap();
    }

    fn animate_cursor(&mut self, cursor: Rectangle) {
        let area = &self.display.bounding_box();
        let cursor_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::new(154, 153, 150))
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        // Update buffer data with a new cursor and position
        cursor
            .draw_styled(&cursor_style, &mut self.frame_buffer)
            .unwrap();
        
        // Draw the buffer to display with the cursor
        self.transfer_region(&cursor);
    }

    #[inline]
    pub fn render_segmented(&mut self, color: Rgb565, position: Point, message: &str) {
        // Reset the buffer to black, but don't draw to the screen yet
        let draw_area = Rectangle::new(position, Size::new(300, 50));
        let _ = &mut self.frame_buffer.fill_solid(&draw_area, Rgb565::BLACK).unwrap();

        let style = SevenSegmentStyleBuilder::new()
            .digit_size(Size::new(30, 50))
            .digit_spacing(10)
            .segment_width(5)
            .segment_color(color)
            .build();

        let text = Text::with_baseline(message, position, style, Baseline::Top);

        // Write time pixel data to the buffer
        let _ = text.draw(&mut self.frame_buffer).unwrap();

        // Finally, draw the buffer to the screen
        self.transfer_region(&draw_area);

    }
}
