pub const START_ROM: usize = 0x200;
const ROM_SIZE: usize = 4096 - START_ROM;

const FONT_DATA: &'static [u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
pub(crate) struct Memory(pub [u8; 4096]);

impl Memory {
    pub fn new() -> Self {
        Memory([0; 4096])
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.0[0..FONT_DATA.len()].copy_from_slice(FONT_DATA);

        let rom_size = bytes.len();
        self.0[START_ROM..START_ROM + rom_size].copy_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Dummy, Fake, Faker};
    use quickcheck_macros::quickcheck;
    use rand::{rngs::StdRng, SeedableRng};

    #[derive(Debug, Clone, Dummy)]
    struct RomFixture {
        #[dummy(faker = "(Faker, 1..3584)")]
        bytes: Vec<u8>,
    }

    impl quickcheck::Arbitrary for RomFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));

            Faker.fake_with_rng(&mut rng)
        }
    }

    #[quickcheck]
    fn test_load_rom(rom: RomFixture) {
        let num_bytes = rom.bytes.len();

        let mut memory = Memory::new();
        memory.load_rom(&rom.bytes);

        assert_eq!(memory.0[START_ROM..START_ROM + num_bytes], rom.bytes);
    }
}
