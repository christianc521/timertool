use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/assets/dice.gif");
    
    let input_path = "./gifs/dice.gif";
    let output_path = "src/assets/dice_rgb565.bin";
    
    if Path::new(input_path).exists() {
        convert_gif_to_rgb565(input_path, output_path)
            .expect("Failed to convert GIF");
    }
}

fn convert_gif_to_rgb565(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use gif::{DecodeOptions, Decoder};
    
    let file = File::open(input)?;
    let mut options = DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    
    let mut decoder = options.read_info(file)?;
    let mut output_file = BufWriter::new(File::create(output)?);
    
    let mut frame_count = 0u32;
    let width = decoder.width();
    let height = decoder.height();
    
    // Process each frame
    while let Some(frame) = decoder.read_next_frame()? {
        let pixels = &frame.buffer;
        
        // Convert RGBA to RGB565 (little-endian for ESP32)
        for chunk in pixels.chunks(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            // chunk[3] is alpha, ignored
            
            let rgb565 = rgb888_to_rgb565(r, g, b);
            
            // Write as little-endian (ESP32 native)
            output_file.write_all(&rgb565.to_le_bytes())?;
        }
        
        frame_count += 1;
    }
    
    println!("cargo:warning=Converted {frame_count} frames, {}x{}", width, height);
    
    Ok(())
}

/// Convert 24-bit RGB to 16-bit RGB565
fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;  // 5 bits
    let g6 = (g >> 2) as u16;  // 6 bits  
    let b5 = (b >> 3) as u16;  // 5 bits
    
    (r5 << 11) | (g6 << 5) | b5
}
