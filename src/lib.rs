pub mod display;
pub mod interpreter;
pub mod keyboard;
mod memory;
mod registers;
pub mod sound;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;
