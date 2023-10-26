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

impl Registers {}
