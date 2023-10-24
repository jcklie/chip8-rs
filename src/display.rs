const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

/// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display with this format:
/// ( 0, 0)   (63, 0)
/// ( 0,31)   (63,31)
pub struct Display([bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]);

impl Display {
    pub fn new() -> Self {
        Display([false; DISPLAY_WIDTH * DISPLAY_HEIGHT])
    }

    pub fn clear(&mut self) {
        for i in &mut self.0 {
            *i = false
        }
    }

    pub fn pixel(&self, x: usize, y: usize) -> bool {
        self.0[self.compute_idx(x, y)]
    }

    /// Xors the pixel at position (`x`, `y`) and returns `true`
    /// if the pixel was cleared.
    pub fn xor_pixel(&mut self, x: usize, y: usize, value: bool) -> bool {
        let idx = self.compute_idx(x, y);
        let last_value = self.0[idx];
        let new_value = last_value ^ value;
        self.0[idx] = new_value;

        last_value && !new_value
    }

    pub fn compute_idx(&self, x: usize, y: usize) -> usize {
        y * self.width() + x
    }

    pub fn pixels(&self) -> &[bool] {
        &self.0
    }

    pub fn width(&self) -> usize {
        DISPLAY_WIDTH
    }

    pub fn height(&self) -> usize {
        DISPLAY_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_pixel() {
        let mut display = Display::new();

        for x in 0..display.width() {
            for y in 0..display.height() {
                display.xor_pixel(x, y, true);

                assert_eq!(display.pixel(x, y), true);

                display.xor_pixel(x, y, true);

                assert_eq!(display.pixel(x, y), false);
            }
        }
    }

    #[test]
    fn test_clear() {
        let mut display = Display::new();

        for x in 0..display.width() {
            for y in 0..display.height() {
                display.xor_pixel(x, y, true);
                assert_eq!(display.pixel(x, y), true);
            }
        }

        display.clear();

        for x in 0..display.width() {
            for y in 0..display.height() {
                assert_eq!(display.pixel(x, y), false);
            }
        }
    }
}
