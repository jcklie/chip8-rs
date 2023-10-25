use std::convert::TryInto;

use crate::{
    display::Display,
    memory::{Memory, START_ROM},
    registers::Registers,
};

pub struct Interpreter {
    registers: Registers,
    memory: Memory,
    display: Display,
}

impl Interpreter {
    pub fn with_rom(bytes: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_rom(bytes);

        let mut registers = Registers::default();
        registers.pc = START_ROM as u16;

        let display = Display::new();

        Interpreter {
            registers,
            memory,
            display,
        }
    }

    pub fn step(&mut self) {
        let pc = self.registers.pc as usize;

        let cur: u16 = u16::from_be_bytes(self.memory.0[pc..pc + 2].try_into().unwrap());

        let first_byte = self.memory.0[pc];
        let second_byte = self.memory.0[pc + 1];

        let first_nibble = ((cur & 0xF000) >> 12) as u8;
        let second_nibble = ((cur & 0x0F00) >> 8) as u8;
        let third_nibble = ((cur & 0x0F0) >> 4) as u8;
        let fourth_nibble = (cur & 0x000F) as u8;

        let bottom_tribble = cur & 0x0FFF;

        // println!("A: {:#01X}, B: {:#01X}, C: {:#01X}, D: {:#01X}", first_nibble, second_nibble, third_nibble, fourth_nibble);

        if cur == 0x00E0 {
            self.handle_clear();
        } else {
            match first_nibble {
                0x1 => {
                    self.handle_jump(bottom_tribble);
                    return;
                }
                0x3 => self.handle_skip_if_equal_immediate(second_nibble as usize, second_byte as u16),
                0x4 => self.handle_skip_if_not_equal_immediate(second_nibble as usize, second_byte as u16),
                0x5 if fourth_nibble == 0 => {
                    self.handle_skip_if_equal_register(second_nibble as usize, third_nibble as usize)
                }
                0x6 => self.handle_load_register_immediate(second_nibble as usize, second_byte as u16),
                0x7 => self.handle_add_register_immediate(second_nibble as usize, second_byte as u16),
                0x8 if fourth_nibble == 0 => {
                    self.handle_load_register_register(second_nibble as usize, third_nibble as usize)
                }
                0xA => self.handle_load_immediate(bottom_tribble),
                0xD => self.handle_draw_sprite(second_nibble, third_nibble, fourth_nibble),

                _ => panic!("Unknown instruction: {:#02x}", cur),
            }
        }

        self.registers.pc += 2;
    }

    fn handle_clear(&mut self) {
        self.display.clear();
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    ///
    /// The interpreter sets the program counter to nnn.
    fn handle_jump(&mut self, n: u16) {
        self.registers.pc = n;
    }

    /// 3xkk - SE Vx, byte
    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn handle_skip_if_equal_immediate(&mut self, x: usize, k: u16) {
        if self.registers.vx[x] == k {
            self.registers.pc += 2;
        }
    }

    /// 4xkk - SNE Vx, byte
    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn handle_skip_if_not_equal_immediate(&mut self, x: usize, k: u16) {
        if self.registers.vx[x] != k {
            self.registers.pc += 2;
        }
    }

    /// 5xy0 - SE Vx, Vy
    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn handle_skip_if_equal_register(&mut self, x: usize, y: usize) {
        if self.registers.vx[x] == self.registers.vx[y] {
            self.registers.pc += 2;
        }
    }

    /// 6xkk - LD Vx, byte
    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    fn handle_load_register_immediate(&mut self, x: usize, k: u16) {
        self.registers.vx[x] = k;
    }

    /// 7xkk - ADD Vx, byte
    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn handle_add_register_immediate(&mut self, x: usize, k: u16) {
        self.registers.vx[x] += k;
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    fn handle_load_register_register(&mut self, x: usize, y: usize) {
        self.registers.vx[x] = self.registers.vx[y];
    }

    /// Annn - LD I, addr
    /// Set I = nnn.
    ///
    /// The value of register I is set to nnn.
    fn handle_load_immediate(&mut self, n: u16) {
        self.registers.i = n;
    }

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes
    /// are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the
    /// existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set
    /// to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it
    /// wraps around to the opposite side of the screen. See instruction 8xy3 for more information on
    /// XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    fn handle_draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        let mut was_cleared = false;

        let mut row: usize = self.registers.vx[y as usize].into();

        for offset in 0..n {
            let idx = self.registers.i as usize + offset as usize;
            let sprite = &self.memory.0[idx];

            let mut mask = 0b1000_0000;

            let mut col: usize = self.registers.vx[x as usize].into();

            for _ in 0..8 {
                let value = (sprite & mask) > 0;
                if self.display.xor_pixel(col, row, value) {
                    was_cleared = true;
                }

                mask >>= 1;

                col += 1;

                print!("{}", if value { "o" } else { " " });
            }

            row += 1;
        }

        if was_cleared {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    pub fn display(&self) -> &Display {
        &self.display
    }
}

#[cfg(test)]
mod tests {
    use super::Interpreter;

    #[test]
    fn test_handle_clear() {}

    #[test]
    fn test_handle_jump() {
        let rom: &[u8] = &[0x17, 0x89];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x789);
    }

    #[test]
    fn test_handle_skip_if_equal_immediate_equal() {
        let rom: &[u8] = &[0x33, 0x42];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x42;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x204);
    }

    #[test]
    fn test_handle_skip_if_equal_immediate_unequal() {
        let rom: &[u8] = &[0x33, 0x42];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x43;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x202);
    }

    #[test]
    fn test_handle_skip_if_not_equal_immediate_equal() {
        let rom: &[u8] = &[0x43, 0x42];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x42;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x202);
    }

    #[test]
    fn test_handle_skip_if_not_equal_immediate_unequal() {
        let rom: &[u8] = &[0x43, 0x42];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x43;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x204);
    }

    #[test]
    fn test_handle_skip_if_not_equal_register_equal() {
        let rom: &[u8] = &[0x53, 0x40];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x42;
        interpreter.registers.vx[4] = 0x42;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x204);
    }

    #[test]
    fn test_handle_skip_if_not_equal_register_unequal() {
        let rom: &[u8] = &[0x53, 0x40];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[3] = 0x42;
        interpreter.registers.vx[4] = 0x23;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x202);
    }

    #[test]
    fn test_handle_load_register_immediate() {
        let rom: &[u8] = &[0x61, 0x23];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.vx[1], 0x23);
    }

    #[test]
    fn test_handle_add_register_immediate() {
        let rom: &[u8] = &[0x73, 0x21, 0x73, 0x10];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();
        interpreter.step();

        assert_eq!(interpreter.registers.vx[3], 0x31);
    }

    #[test]
    fn test_handle_load_register_register() {
        let rom: &[u8] = &[0x8A, 0xC0];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.registers.vx[0xC] = 0x23;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[0xa], 0x23);
    }

    #[test]
    fn test_handle_load_immediate() {
        let rom: &[u8] = &[0xA6, 0x78];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.i, 0x678);
    }
}
