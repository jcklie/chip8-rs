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

        let _first_byte = self.memory.0[pc];
        let second_byte = self.memory.0[pc + 1];

        let first_nibble = ((cur & 0xF000) >> 12) as u8;
        let second_nibble = ((cur & 0x0F00) >> 8) as u8;
        let third_nibble = ((cur & 0x0F0) >> 4) as u8;
        let fourth_nibble = (cur & 0x000F) as u8;

        let bottom_tribble = cur & 0x0FFF;

        // println!("A: {:#01X}, B: {:#01X}, C: {:#01X}, D: {:#01X}", first_nibble, second_nibble, third_nibble, fourth_nibble);

        if cur == 0x00E0 {
            self.handle_clear();
        } else if cur == 0x00EE {
            self.handle_ret();
        } else {
            match first_nibble {
                0x1 => {
                    self.handle_jump(bottom_tribble);
                    return;
                }
                0x2 => {
                    self.handle_call(bottom_tribble);
                    return;
                }
                0x3 => self.handle_skip_if_equal_immediate(second_nibble as usize, second_byte),
                0x4 => self.handle_skip_if_not_equal_immediate(second_nibble as usize, second_byte),
                0x5 if fourth_nibble == 0 => {
                    self.handle_skip_if_equal_register(second_nibble as usize, third_nibble as usize)
                }
                0x6 => self.handle_load_register_immediate(second_nibble as usize, second_byte),
                0x7 => self.handle_add_register_immediate(second_nibble as usize, second_byte),
                0x8 if fourth_nibble == 0 => {
                    self.handle_load_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 1 => {
                    self.handle_or_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 2 => {
                    self.handle_and_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 3 => {
                    self.handle_xor_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 4 => {
                    self.handle_add_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 5 => {
                    self.handle_sub_register_register(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 6 => {
                    self.handle_shift_right_register_one(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 7 => {
                    self.handle_sub_register_register_negated(second_nibble as usize, third_nibble as usize)
                }
                0x8 if fourth_nibble == 0xE => {
                    self.handle_shift_left_register_one(second_nibble as usize, third_nibble as usize)
                }
                0x9 if fourth_nibble == 0 => {
                    self.handle_skip_if_not_equal_register(second_nibble as usize, third_nibble as usize)
                }
                0xA => self.handle_load_immediate(bottom_tribble),
                0xD => self.handle_draw_sprite(second_nibble, third_nibble, fourth_nibble),

                _ => eprintln!("Unknown instruction: {:#02x}", cur),
            }
        }

        self.registers.pc += 2;
    }

    fn handle_clear(&mut self) {
        self.display.clear();
    }

    /// 00EE - RET
    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn handle_ret(&mut self) {
        self.registers.pop();
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    ///
    /// The interpreter sets the program counter to nnn.
    fn handle_jump(&mut self, n: u16) {
        self.registers.pc = n;
    }

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn handle_call(&mut self, n: u16) {
        self.registers.push(n);
    }

    /// 3xkk - SE Vx, byte
    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn handle_skip_if_equal_immediate(&mut self, x: usize, k: u8) {
        if self.registers.vx[x] == k {
            self.registers.pc += 2;
        }
    }

    /// 4xkk - SNE Vx, byte
    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn handle_skip_if_not_equal_immediate(&mut self, x: usize, k: u8) {
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
    fn handle_load_register_immediate(&mut self, x: usize, k: u8) {
        self.registers.vx[x] = k;
    }

    /// 7xkk - ADD Vx, byte
    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn handle_add_register_immediate(&mut self, x: usize, k: u8) {
        let result = self.registers.vx[x].wrapping_add(k);
        self.registers.vx[x] = result;
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    fn handle_load_register_register(&mut self, x: usize, y: usize) {
        self.registers.vx[x] = self.registers.vx[y];
    }

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    fn handle_or_register_register(&mut self, x: usize, y: usize) {
        self.registers.vx[x] |= self.registers.vx[y];
    }

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    fn handle_and_register_register(&mut self, x: usize, y: usize) {
        self.registers.vx[x] &= self.registers.vx[y];
    }

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise XOR on the values of Vx and Vy, then stores the result in Vx.
    fn handle_xor_register_register(&mut self, x: usize, y: usize) {
        self.registers.vx[x] ^= self.registers.vx[y];
    }

    /// 8xy4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry.
    ///
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits
    /// (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn handle_add_register_register(&mut self, x: usize, y: usize) {
        let a = self.registers.vx[x];
        let b = self.registers.vx[y];

        let (result, overflow) = a.overflowing_add(b);
        self.registers.vx[x] = result;

        if overflow {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    /// 8xy5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    ///
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn handle_sub_register_register(&mut self, x: usize, y: usize) {
        let a = self.registers.vx[x];
        let b = self.registers.vx[y];

        let result = a.wrapping_sub(b);

        self.registers.vx[x] = result;

        if a > b {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx SHR 1.
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn handle_shift_right_register_one(&mut self, x: usize, _y: usize) {
        let a = self.registers.vx[x];

        let underflow = a & 1 == 1;
        let result = a >> 1;

        self.registers.vx[x] = result;

        if underflow {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    ///
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn handle_sub_register_register_negated(&mut self, x: usize, y: usize) {
        let a = self.registers.vx[x];
        let b = self.registers.vx[y];

        let result = b.wrapping_sub(a);

        self.registers.vx[x] = result;

        if b > a {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn handle_shift_left_register_one(&mut self, x: usize, _y: usize) {
        let a = self.registers.vx[x];

        let overflow = a & 0b1000_0000 > 1;
        let result = a << 1;

        self.registers.vx[x] = result;

        if overflow {
            self.registers.vx[0xF] = 1;
        } else {
            self.registers.vx[0xF] = 0;
        }
    }

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy.
    ///
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn handle_skip_if_not_equal_register(&mut self, x: usize, y: usize) {
        if self.registers.vx[x] != self.registers.vx[y] {
            self.registers.pc += 2;
        }
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
    use test_case::test_case;

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
    fn test_handle_push() {
        let rom: &[u8] = &[0x21, 0x23];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();
        assert_eq!(interpreter.registers.sp, 1);
        assert_eq!(interpreter.registers.pc, 0x123);
    }

    #[test]
    fn test_handle_pop() {
        let rom: &[u8] = &[0x22, 0x06, 0x00, 0xE0, 0x00, 0xE0, 0x00, 0xEE];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();
        assert_eq!(interpreter.registers.sp, 1);
        assert_eq!(interpreter.registers.pc, 0x206);

        interpreter.step();
        assert_eq!(interpreter.registers.sp, 0);
        assert_eq!(interpreter.registers.pc, 0x202);
    }

    #[test_case(3 , 15, 15, 0x204; "SE: vx equals k")]
    #[test_case(7, 0x42, 0x23, 0x202 ; "SE: vx does not equal k")]
    fn test_handle_skip_if_equal_immediate(x: u8, vx: u8, k: u8, pc: u16) {
        let rom: &[u8] = &[0x30 | x, k];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
    }

    #[test_case(0xA , 0x18, 0x18, 0x202; "SNE: vx equals k")]
    #[test_case(0xB, 0x13, 0x55, 0x204 ; "SNE: vx does not equal k")]
    fn test_handle_skip_if_not_equal_immediate(x: u8, vx: u8, k: u8, pc: u16) {
        let rom: &[u8] = &[0x40 | x, k];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
    }

    #[test_case(0xA , 0x0, 0x18, 0x18, 0x204; "SE: vx equals vy")]
    #[test_case(0x7, 0x5, 1, 0x55, 0x202 ; "SE: vx does not equal vy")]
    fn test_handle_skip_if_equal_register(x: u8, y: u8, vx: u8, vy: u8, pc: u16) {
        let rom: &[u8] = &[0x50 | x, y << 4];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;
        interpreter.registers.vx[y as usize] = vy;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
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

        assert_eq!(interpreter.registers.vx[0xA], 0x23);
    }

    #[test]
    fn test_handle_or_register_register() {
        let rom: &[u8] = &[0x8B, 0xD1];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.registers.vx[0xB] = 0x23;
        interpreter.registers.vx[0xD] = 0x42;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[0xB], 0x63);
    }

    #[test]
    fn test_handle_and_register_register() {
        let rom: &[u8] = &[0x8E, 0x12];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.registers.vx[0xE] = 0x23;
        interpreter.registers.vx[0x1] = 0x42;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[0xE], 0x2);
    }

    #[test]
    fn test_handle_xor_register_register() {
        let rom: &[u8] = &[0x89, 0x73];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.registers.vx[0x9] = 0x15;
        interpreter.registers.vx[0x7] = 0x37;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[0x9], 0x22);
    }

    #[test_case(0xB , 0x3, 5, 3, 8, 0; "ADD: vx + vy - No overflow")]
    #[test_case(0x2, 0x9, 0xFA, 0x13, 0xD, 1 ; "ADD: vx + vy - Overflow")]
    #[test_case(0xF, 0x0, 0xAA, 0xBB, 1, 1 ; "ADD: vx + vy - Target VF + Overflow")]
    #[test_case(0xF, 0x7, 17, 58, 0, 0 ; "ADD: vx + vy - Target VF + No Overflow")]
    fn test_handle_add_register_register(x: u8, y: u8, vx: u8, vy: u8, result: u8, carry: u8) {
        let rom: &[u8] = &[0x80 | x, (y << 4) | 0x4];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;
        interpreter.registers.vx[y as usize] = vy;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[x as usize], result, "Result wrong");
        assert_eq!(interpreter.registers.vx[0xF], carry, "Carry wrong");
    }

    #[test_case(0xC , 0x2, 25, 12, 13, 1; "SUB: vx - vy - No Underflow")]
    #[test_case(0xD, 0x4, 0x13, 0x15, 0b11111110, 0 ; "SUB: vx - vy - Underflow")]
    #[test_case(0xF, 0x0, 5, 7, 0, 0 ; "SUB: vx - vy - Target VF - Underflow")]
    #[test_case(0xF, 0xE, 7, 5, 1, 1 ; "SUB: vx - vy - Target VF - No Underflow")]
    fn test_handle_sub_register_register(x: u8, y: u8, vx: u8, vy: u8, result: u8, underflow: u8) {
        let rom: &[u8] = &[0x80 | x, (y << 4) | 0x5];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;
        interpreter.registers.vx[y as usize] = vy;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[x as usize], result, "Result wrong");
        assert_eq!(interpreter.registers.vx[0xF], underflow, "Underflow wrong");
    }

    #[test_case(0x0 , 0x2, 8, 4, 0; "SHR: vx, {vy} - No Underflow")]
    #[test_case(0xE, 0xA, 0b10110011, 0b01011001, 1 ; "SHR: vx, {vy} - Underflow")]
    #[test_case(0xF, 0x2, 0b101, 1, 1 ; "SHR: vx, {vy} - Target VF - Underflow")]
    #[test_case(0xF, 0x3, 0b110, 0, 0 ; "SHR: vx, {vy} - Target VF - No Underflow")]
    fn test_handle_shift_right_register_one(x: u8, y: u8, vx: u8, result: u8, underflow: u8) {
        let rom: &[u8] = &[0x80 | x, (y << 4) | 0x6];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[x as usize], result, "Result wrong");
        assert_eq!(interpreter.registers.vx[0xF], underflow, "Underflow wrong");
    }

    #[test_case(0xD, 0x4, 0x13, 0x15, 0x2, 1 ; "SUBN: vy - vx - No Underflow")]
    #[test_case(0xC , 0x2, 50, 25, 0b1110_0111, 0; "SUBN: vy - vx - Underflow")]
    #[test_case(0xF, 0xE, 7, 5, 0, 0 ; "SUBN: vy - vx - Target VF - Underflow")]
    #[test_case(0xF, 0x0, 5, 7, 1, 1 ; "SUBN: vy - vx - Target VF - No Underflow")]
    fn test_handle_sub_register_register_negated(x: u8, y: u8, vx: u8, vy: u8, result: u8, underflow: u8) {
        let rom: &[u8] = &[0x80 | x, (y << 4) | 0x7];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;
        interpreter.registers.vx[y as usize] = vy;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[x as usize], result, "Result wrong");
        assert_eq!(interpreter.registers.vx[0xF], underflow, "Underflow wrong");
    }

    #[test_case(0x5 , 0x3, 8, 16, 0; "SHL: vx, {vy} - No Overflow")]
    #[test_case(0xA, 0xF, 0b1011_0011, 0b0110_0110, 1 ; "SHL: vx, {vy} - Overflow")]
    #[test_case(0xF, 0xA, 0xFE, 1, 1 ; "SHL: vx, {vy} - Target VF - Overflow")]
    #[test_case(0xF, 0x7, 0b110, 0, 0 ; "SHL: vx, {vy} - Target VF - No Overflow")]
    fn test_handle_shift_left_register_one(x: u8, y: u8, vx: u8, result: u8, overflow: u8) {
        let rom: &[u8] = &[0x80 | x, (y << 4) | 0xE];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[x as usize], result, "Result wrong");
        assert_eq!(interpreter.registers.vx[0xF], overflow, "Overflow wrong");
    }

    #[test_case(0xA , 0x0, 0x18, 0x18, 0x202; "SNE: vx equals vy")]
    #[test_case(0x7, 0x5, 1, 0x55, 0x204 ; "SNE: vx does not equal vy")]
    fn test_handle_skip_if_not_equal_register(x: u8, y: u8, vx: u8, vy: u8, pc: u16) {
        let rom: &[u8] = &[0x90 | x, y << 4];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;
        interpreter.registers.vx[y as usize] = vy;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
    }

    #[test]
    fn test_handle_load_immediate() {
        let rom: &[u8] = &[0xA6, 0x78];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.i, 0x678);
    }
}
