#[derive(Debug, Default)]
pub(crate) struct Registers {
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    vf: u8,

    i: u16,
    /// The program counter (PC) should be 16-bit, and is used to store the currently executing address.
    pc: u16,
    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    sp: u16,

    delay: u8,
    sound: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    stack: [u16; 16]
}

impl Registers {

}