use crate::{registers::Registers, memory::Memory};

pub struct Interpreter {
    registers: Registers,
    memory: Memory
}

impl Interpreter {
    pub fn with_rom(bytes: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_rom(bytes);

        Interpreter {
            registers: Registers::default(),
            memory
        }
    }

    pub fn step(&mut self) {

    }

}