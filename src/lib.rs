pub mod display;
pub mod interpreter;
mod memory;
mod registers;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;
