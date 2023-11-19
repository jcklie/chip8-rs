use std::convert::TryInto;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::{
    display::Display,
    keyboard::Keyboard,
    memory::{Memory, START_ROM},
    registers::Registers,
};

pub struct Interpreter {
    registers: Registers,
    memory: Memory,
    display: Display,
    keyboard: Keyboard,
    rng: ChaCha8Rng,
}

impl Interpreter {
    pub fn with_rom(bytes: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_rom(bytes);

        let mut registers = Registers::default();
        registers.pc = START_ROM as u16;

        let display = Display::new();
        let keyboard = Keyboard::new();

        let rng = ChaCha8Rng::seed_from_u64(09122022);
        Interpreter {
            registers,
            memory,
            display,
            keyboard,
            rng,
        }
    }

    pub fn step(&mut self) {
        self.registers.delay = self.registers.delay.saturating_sub(1);
        self.registers.sound = self.registers.sound.saturating_sub(1);

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
                0x0 | 0x1 => {
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
                0xA => self.handle_load_immediate_into_i(bottom_tribble),
                0xB => {
                    self.handle_jump_relative(bottom_tribble);
                    return;
                }
                0xC => self.handle_random(second_nibble as usize, second_byte),
                0xD => self.handle_draw_sprite(second_nibble, third_nibble, fourth_nibble),
                0xE if second_byte == 0x9E => self.handle_skip_if_key_pressed(second_nibble as usize),
                0xE if second_byte == 0xA1 => self.handle_skip_if_key_not_pressed(second_nibble as usize),
                0xF if second_byte == 0x07 => self.handle_store_delay_timer_register(second_nibble as usize),
                0xF if second_byte == 0x0A => {
                    self.handle_wait_for_keypress(second_nibble as usize);
                    return;
                }
                0xF if second_byte == 0x15 => self.handle_load_delay_timer_register(second_nibble as usize),
                0xF if second_byte == 0x18 => self.handle_load_sound_timer_register(second_nibble as usize),
                0xF if second_byte == 0x29 => self.handle_load_digit_sprite_location(second_nibble as usize),
                0xF if second_byte == 0x1E => self.handle_add_i_register(second_nibble as usize),
                0xF if second_byte == 0x33 => self.handle_load_bcd(second_nibble as usize),
                0xF if second_byte == 0x55 => self.handle_store_registers_in_memory(second_nibble as usize),
                0xF if second_byte == 0x65 => self.handle_load_registers_from_memory(second_nibble as usize),
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
    fn handle_load_immediate_into_i(&mut self, n: u16) {
        self.registers.i = n;
    }

    /// Bnnn - JP V0, addr
    /// Jump to location nnn + V0.
    ///
    /// The program counter is set to nnn plus the value of V0.
    fn handle_jump_relative(&mut self, n: u16) {
        self.registers.pc = n.wrapping_add(self.registers.vx[0].into());
    }

    /// Cxkk - RND Vx, byte
    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    fn handle_random(&mut self, x: usize, k: u8) {
        let v = self.rng.gen_range(0..=255);
        self.registers.vx[x] = v & k;
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

    /// Ex9E - SKP Vx
    /// Skip next instruction if key with the value of Vx is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    fn handle_skip_if_key_pressed(&mut self, x: usize) {
        let keycode = self.registers.vx[x];

        if self.keyboard.is_pressed(keycode) {
            self.registers.pc += 2;
        }
    }

    /// ExA1 - SKNP Vx
    /// Skip next instruction if key with the value of Vx is not pressed.
    ///     
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    fn handle_skip_if_key_not_pressed(&mut self, x: usize) {
        let keycode = self.registers.vx[x];

        if !self.keyboard.is_pressed(keycode) {
            self.registers.pc += 2;
        }
    }

    /// Fx07 - LD Vx, DT
    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    fn handle_store_delay_timer_register(&mut self, x: usize) {
        self.registers.vx[x] = self.registers.delay;
    }

    /// Fx0A - LD Vx, K
    /// Wait for a key press, store the value of the key in Vx.
    ///
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn handle_wait_for_keypress(&mut self, x: usize) {
        if let Some(keycode) = self.keyboard.wait_for_keypress() {
            self.registers.vx[x] = keycode;
            self.registers.pc += 2;
        }
    }

    /// Fx15 - LD DT, Vx
    /// Set delay timer = Vx.
    ///
    /// DT is set equal to the value of Vx.
    fn handle_load_delay_timer_register(&mut self, x: usize) {
        self.registers.delay = self.registers.vx[x]
    }

    /// Fx18 - LD ST, Vx
    /// Set sound timer = Vx.
    ///
    /// ST is set equal to the value of Vx.
    fn handle_load_sound_timer_register(&mut self, x: usize) {
        self.registers.sound = self.registers.vx[x]
    }

    /// Fx29 - LD F, Vx
    /// Set I = location of sprite for digit Vx.
    ///
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    fn handle_load_digit_sprite_location(&mut self, x: usize) {
        self.registers.i = (x as u16).wrapping_mul(5);
    }

    /// Fx1E - ADD I, Vx
    /// Set I = I + Vx.
    ///
    /// The values of I and Vx are added, and the results are stored in I.
    fn handle_add_i_register(&mut self, x: usize) {
        let result = self.registers.i.wrapping_add(self.registers.vx[x].into());
        self.registers.i = result;
    }

    /// Fx33 - LD B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn handle_load_bcd(&mut self, x: usize) {
        let i = self.registers.i as usize;
        let vx = self.registers.vx[x];

        self.memory.0[i + 0] = vx / 100;
        self.memory.0[i + 1] = (vx % 100) / 10;
        self.memory.0[i + 2] = vx % 10;
    }

    /// Fx55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    ///
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    fn handle_store_registers_in_memory(&mut self, x: usize) {
        let i = self.registers.i as usize;
        for offset in 0..=x {
            self.memory.0[i + offset] = self.registers.vx[offset];
        }
    }

    /// Fx65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn handle_load_registers_from_memory(&mut self, x: usize) {
        let i = self.registers.i as usize;
        for offset in 0..=x {
            self.registers.vx[offset] = self.memory.0[i + offset];
        }
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub fn sound_timer_active(&self) -> bool {
        self.registers.sound > 0
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
    fn test_handle_load_immediate_into_i() {
        let rom: &[u8] = &[0xA6, 0x78];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.i, 0x678);
    }

    #[test]
    fn test_handle_jump_relative() {
        let rom: &[u8] = &[0xB6, 0x78];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[0] = 0x13;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x678 + 0x13);
    }

    #[test]
    fn test_handle_random() {
        let rom: &[u8] = &[0xC1, 0xFF];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_ne!(interpreter.registers.vx[1], 0);
    }

    #[test_case(0x3, 0x5, Some(0x5), 0x204; "SKP Vx: wanted key is pressed")]
    #[test_case(0xE, 0x1, None,  0x202; "SKP Vx: no key pressed")]
    #[test_case(0x7, 0xB, Some(0xE),  0x202; "SKP Vx: different pressed")]
    fn test_handle_skip_if_key_pressed(x: u8, vx: u8, key: Option<u8>, pc: u16) {
        let rom: &[u8] = &[0xE0 | x, 0x9E];
        let mut interpreter = Interpreter::with_rom(rom);

        if let Some(keycode) = key {
            interpreter.keyboard_mut().press_key(keycode)
        }

        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
    }

    #[test_case(0x3, 0x5, Some(0x5), 0x202; "SKNP Vx: specified key is pressed")]
    #[test_case(0xE, 0x1, None,  0x204; "SKNP Vx: no key pressed")]
    #[test_case(0x7, 0xB, Some(0xE),  0x204; "SKNP Vx: different pressed")]
    fn test_handle_skip_if_key_not_pressed(x: u8, vx: u8, key: Option<u8>, pc: u16) {
        let rom: &[u8] = &[0xE0 | x, 0xA1];
        let mut interpreter = Interpreter::with_rom(rom);

        if let Some(keycode) = key {
            interpreter.keyboard_mut().press_key(keycode)
        }

        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
    }

    #[test_case(0x3, None, 0x200, 0; "LD Vx, K: not pressed")]
    #[test_case(0xE, Some(0xA),  0x202, 0xA; "LD Vx, K: pressed")]
    fn test_handle_wait_for_keypress(x: u8, key: Option<u8>, pc: u16, vx: u8) {
        let rom: &[u8] = &[0xF0 | x, 0x0A, 0xF0 | x, 0x0A];
        let mut interpreter = Interpreter::with_rom(rom);

        // Press before waiting, should not complete the wait
        if let Some(keycode) = key {
            interpreter.keyboard_mut().press_key(keycode);
            interpreter.keyboard_mut().release_key(keycode);
        }

        // Wait for keypress
        interpreter.step();

        assert_eq!(interpreter.registers.pc, 0x200);
        assert_eq!(interpreter.registers.vx[x as usize], 0);

        // If we press now, then it should complete
        if let Some(keycode) = key {
            interpreter.keyboard_mut().press_key(keycode);
            interpreter.keyboard_mut().release_key(keycode);
        }

        interpreter.step();

        assert_eq!(interpreter.registers.pc, pc);
        assert_eq!(interpreter.registers.vx[x as usize], vx);
    }

    #[test]
    fn test_load_delay_timer_register() {
        let rom: &[u8] = &[0xFA, 0x15];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[0xA] = 23;

        interpreter.step();

        assert_eq!(interpreter.registers.vx[0xA], 23);
    }

    #[test_case(0x5 , 5, 7, 12; "ADD i, vx: no overflow")]
    #[test_case(0x5 , 0xFA, 0xFFFA, 0xF4; "ADD i, vx: overflow")]
    fn test_handle_add_i_register(x: u8, vx: u8, i: u16, result: u16) {
        let rom: &[u8] = &[0xF0 | x, 0x1E];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.registers.i = i;
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(interpreter.registers.i, result);
    }

    #[test]
    fn handle_load_digit_sprite_location() {
        let rom: &[u8] = &[0xF7, 0x29];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.i, 0x7 * 5);
    }

    #[test_case(0x5 , 223, 2, 2, 3; "BCD: xyz")]
    #[test_case(0x5 , 109, 1, 0, 9; "BCD: x0z")]
    #[test_case(0x3 , 42, 0, 4, 2; "BCD: yz")]
    #[test_case(0xA , 7, 0, 0, 7; "BCD: z")]
    fn test_handle_bcd(x: u8, vx: u8, hundreds: u8, tens: u8, ones: u8) {
        let rom: &[u8] = &[0xF0 | x, 0x33];
        let mut interpreter = Interpreter::with_rom(rom);
        interpreter.registers.vx[x as usize] = vx;

        interpreter.step();

        assert_eq!(
            interpreter.memory.0[interpreter.registers.i as usize + 0],
            hundreds,
            "xyz"
        );
        assert_eq!(interpreter.memory.0[interpreter.registers.i as usize + 1], tens, "yz");
        assert_eq!(interpreter.memory.0[interpreter.registers.i as usize + 2], ones, "z");
    }

    #[test]
    fn test_handle_store_registers_in_memory() {
        let values: Vec<u8> = vec![116, 58, 224, 135, 225, 142, 236, 47, 66, 29, 230, 171, 127, 21, 11, 147];

        for x in 0..16 {
            let rom: &[u8] = &[0xF0 | x, 0x55];
            let mut interpreter = Interpreter::with_rom(rom);
            interpreter.registers.i = 0x400;

            for i in 0..16 {
                interpreter.registers.vx[i] = values[i];
            }

            interpreter.step();

            for i in 0..=x as usize {
                assert_eq!(interpreter.memory.0[interpreter.registers.i as usize + i], values[i]);
            }

            for i in (x + 1) as usize..16 {
                assert_eq!(interpreter.memory.0[interpreter.registers.i as usize + i], 0);
            }
        }
    }

    #[test]
    fn test_handle_load_registers_from_memory() {
        let values: Vec<u8> = vec![116, 58, 224, 135, 225, 142, 236, 47, 66, 29, 230, 171, 127, 21, 11, 147];

        for x in 0..16 {
            let rom: &[u8] = &[0xF0 | x, 0x65];
            let mut interpreter = Interpreter::with_rom(rom);
            interpreter.registers.i = 0x400;

            for i in 0..16 {
                interpreter.memory.0[interpreter.registers.i as usize + i] = values[i];
            }

            interpreter.step();

            for i in 0..=x as usize {
                assert_eq!(interpreter.registers.vx[i], values[i]);
            }

            for i in (x + 1) as usize..16 {
                assert_eq!(interpreter.registers.vx[i], 0);
            }
        }
    }

    #[test]
    fn handle_load_registers_from_memory() {
        let rom: &[u8] = &[0xA6, 0x78];
        let mut interpreter = Interpreter::with_rom(rom);

        interpreter.step();

        assert_eq!(interpreter.registers.i, 0x678);
    }
}
