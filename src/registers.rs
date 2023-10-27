#[derive(Debug, Default)]
pub(crate) struct Registers {
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    pub vx: [u8; 16],

    /// There is also a 16-bit register called I. This register is generally used to store memory addresses,
    /// so only the lowest (rightmost) 12 bits are usually used.
    pub i: u16,
    /// The program counter (PC) should be 16-bit, and is used to store the currently executing address.
    pub pc: u16,
    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    pub sp: u8,

    pub delay: u8,
    pub sound: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to
    /// when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    pub stack: [u16; 16],
}

impl Registers {
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    pub fn push(&mut self, n: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = n;
    }

    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    pub fn pop(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::Registers;

    #[test]
    fn test_push_pop() {
        let mut registers = Registers::default();
        registers.pc = 0x42;

        registers.push(0x23);
        assert_eq!(registers.sp, 1);
        assert_eq!(registers.pc, 0x23);

        registers.push(0x77);
        assert_eq!(registers.sp, 2);
        assert_eq!(registers.pc, 0x77);

        registers.pop();
        assert_eq!(registers.sp, 1);
        assert_eq!(registers.pc, 0x23);

        registers.pop();
        assert_eq!(registers.sp, 0);
        assert_eq!(registers.pc, 0x42);
    }
}
