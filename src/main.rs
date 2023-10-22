use std::path::PathBuf;

use clap::{Parser};

use chip8::interpreter::Interpreter;
use chip8::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path of the rom to load
    #[arg(short, long, value_name = "FILE")]
    rom_path: PathBuf

}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bytes = std::fs::read(cli.rom_path)?; 

    let mut interpreter = Interpreter::with_rom(&bytes);
    interpreter.step();

    Ok(())
}
