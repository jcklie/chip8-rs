use std::path::PathBuf;

use clap::Parser;

use minifb::{Key, Scale, Window, WindowOptions};

use chip8::interpreter::Interpreter;
use chip8::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path of the rom to load
    #[arg(short, long, value_name = "FILE")]
    rom_path: PathBuf,
}

fn run_rom(bytes: &[u8]) -> Result<()> {
    let mut interpreter = Interpreter::with_rom(&bytes);

    let width = interpreter.display().width();
    let height = interpreter.display().height();
    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut opts = WindowOptions::default();
    opts.scale = Scale::FitScreen;

    let mut window = Window::new("Chip-8 - ESC to exit", width, height, opts).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, p) in buffer.iter_mut().zip(interpreter.display().pixels()) {
            *i = if *p { 0xFFFFFF } else { 0 };
        }

        window.update_with_buffer(&buffer, width, height)?;

        interpreter.step();
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bytes = std::fs::read(cli.rom_path)?;

    run_rom(&bytes)?;

    Ok(())
}
