pub const START_ROM: usize = 0x200;
const ROM_SIZE: usize = 4096 - START_ROM;

#[derive(Debug)]
pub(crate) struct Memory(pub [u8; 4096]);

impl Memory {
    pub fn new() -> Self {
        Memory([0; 4096])
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        let num_bytes = bytes.len();
        self.0[START_ROM..START_ROM + num_bytes].copy_from_slice(bytes);
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
