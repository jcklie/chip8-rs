use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;

use minifb::{InputCallback, Key, Scale, Window, WindowOptions};

use chip8::interpreter::Interpreter;
use chip8::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path of the rom to load
    #[arg(short, long, value_name = "FILE")]
    rom_path: PathBuf,
}

struct KeyCharCallback {
    keycode: Option<u8>,
    keymap: HashMap<Key, u8>,
}

impl KeyCharCallback {
    fn new() -> Self {
        let keymap: HashMap<Key, u8> = HashMap::from([
            (Key::Key1, 0x0),
            (Key::Key2, 0x1),
            (Key::Key3, 0x2),
            (Key::Key4, 0x3),
            (Key::Q, 0x4),
            (Key::W, 0x5),
            (Key::E, 0x6),
            (Key::R, 0x7),
            (Key::A, 0x8),
            (Key::S, 0x9),
            (Key::D, 0xA),
            (Key::F, 0xB),
            (Key::Y, 0xC),
            (Key::X, 0xD),
            (Key::C, 0xE),
            (Key::V, 0xF),
        ]);

        Self { keycode: None, keymap }
    }
}

impl InputCallback for KeyCharCallback {
    fn add_char(&mut self, c: u32) {}

    fn set_key_state(&mut self, key: Key, state: bool) {
        if let Some(keycode) = self.keymap.get(&key) {
            // New key is pressed, replace current key
            if state {
                self.keycode = Some(*keycode)
            }
            // Keycode is the same, but key state is false, thus it is release
            else if self.keycode == Some(*keycode) {
                self.keycode = None
            }
        }
    }
}

fn run_rom(bytes: &[u8]) -> Result<()> {
    let keymap: HashMap<Key, u8> = HashMap::from([
        (Key::Key1, 0x0),
        (Key::Key2, 0x1),
        (Key::Key3, 0x2),
        (Key::Key4, 0x3),
        (Key::Q, 0x4),
        (Key::W, 0x5),
        (Key::E, 0x6),
        (Key::R, 0x7),
        (Key::A, 0x8),
        (Key::S, 0x9),
        (Key::D, 0xA),
        (Key::F, 0xB),
        (Key::Y, 0xC),
        (Key::X, 0xD),
        (Key::C, 0xE),
        (Key::V, 0xF),
    ]);

    let mut interpreter = Interpreter::with_rom(bytes);

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
    window.topmost(true);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        interpreter.keyboard_mut().pressed_key = None;

        for key in window.get_keys().iter() {
            // We consider the first key to match the currently pressed key, as CHIP-8 has no concept
            // of multiple keys pressed at the same time.
            if let Some(keycode) = keymap.get(key) {
                interpreter.keyboard_mut().pressed_key = Some(*keycode);
                break;
            }
        }

        interpreter.step();

        for (i, p) in buffer.iter_mut().zip(interpreter.display().pixels()) {
            *i = if *p { 0xFFFFFF } else { 0 };
        }

        window.update_with_buffer(&buffer, width, height)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bytes = std::fs::read(cli.rom_path)?;

    run_rom(&bytes)?;

    Ok(())
}
