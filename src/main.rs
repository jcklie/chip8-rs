use std::collections::HashMap;
use std::path::PathBuf;

use chip8::sound::SquareWave;
use clap::Parser;

use chip8::interpreter::Interpreter;
use chip8::Result;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path of the rom to load
    #[arg(short, long, value_name = "FILE")]
    rom_path: PathBuf,
}

fn run_rom(bytes: &[u8]) -> Result<()> {
    let fps = 500;

    let keymap: HashMap<Keycode, u8> = HashMap::from([
        (Keycode::Num1, 0x1),
        (Keycode::Num2, 0x2),
        (Keycode::Num3, 0x3),
        (Keycode::Num4, 0xC),
        (Keycode::Q, 0x4),
        (Keycode::W, 0x5),
        (Keycode::E, 0x6),
        (Keycode::R, 0xD),
        (Keycode::A, 0x7),
        (Keycode::S, 0x8),
        (Keycode::D, 0x9),
        (Keycode::F, 0xE),
        (Keycode::Y, 0xA),
        (Keycode::Z, 0xA),
        (Keycode::X, 0x0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ]);

    let mut interpreter = Interpreter::with_rom(bytes);

    let scale = 32;

    let width = interpreter.display().width() as u32;
    let height = interpreter.display().height() as u32;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", width * scale, height * scale)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Audio
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.05,
            }
        })
        .unwrap();

    loop {
        if interpreter.sound_timer_active() {
            device.resume();
        } else {
            device.pause();
        }

        // Input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                Event::KeyDown {
                    keycode: Some(keycode), ..
                } if keymap.contains_key(&keycode) => {
                    let key = keymap.get(&keycode).expect("Already checked contains");
                    interpreter.keyboard_mut().press_key(*key);
                }
                Event::KeyUp {
                    keycode: Some(keycode), ..
                } if keymap.contains_key(&keycode) => {
                    let key = keymap.get(&keycode).expect("Already checked contains");
                    interpreter.keyboard_mut().release_key(*key);
                }
                _ => {}
            }
        }

        // Update
        interpreter.step();

        // Draw
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.set_scale(scale as f32, scale as f32).unwrap();
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for (idx, pixel) in interpreter.display().pixels().iter().enumerate() {
            let idx = idx as u32;

            let x = idx % width;
            let y = idx / width;

            if *pixel {
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bytes = std::fs::read(cli.rom_path)?;

    run_rom(&bytes)?;

    Ok(())
}
