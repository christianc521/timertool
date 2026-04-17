use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_simulator::{
    OutputSettingsBuilder,
    SimulatorEvent,
    Window,
};
use timetool_v2::tft::TFT;

fn main() {
    let mut tft = TFT::new_simulator();

    let output_settings = OutputSettingsBuilder::new()
        .scale(1)
        .build();
    let mut window = Window::new("Timetool Simulator", &output_settings);

    window.update(&tft.display);

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    println!("Key pressed: {:?}", keycode);
                }
                _ => {}
            }
        }

        if tft.playing_animation {
            tft.render_next_frame();
            window.update(&tft.display);

            // Throttle to ~15 FPS to match hardware
            std::thread::sleep(std::time::Duration::from_millis(1000 / 15));
        }
    }
}
