use std::fmt::{format, Display};

pub struct Keyboard {
    pressed_keys: [bool; 16],
    most_recent_key: Option<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: [false; 16],
            most_recent_key: None,
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.pressed_keys[key as usize]
    }

    pub fn press_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = true;

        self.most_recent_key = Some(key)
    }

    pub fn release_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = false;
        self.most_recent_key = None;
    }

    pub fn most_recent_key(&self) -> Option<u8> {
        self.most_recent_key
    }
}

impl Display for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pressed_keys.map(|k| if k { "o" } else { " " }).join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pressed() {
        let mut keyboard = Keyboard::new();

        let key: u8 = 0x4;
        keyboard.pressed_keys[key as usize] = true;

        assert!(keyboard.is_pressed(key));
    }

    #[test]
    fn test_press_key() {
        let mut keyboard = Keyboard::new();

        let key: u8 = 0xA;
        keyboard.press_key(key);

        assert!(keyboard.pressed_keys[key as usize]);
        assert_eq!(keyboard.most_recent_key, Some(key));
    }

    #[test]
    fn test_release_key() {
        let mut keyboard = Keyboard::new();

        let key: u8 = 0xE;
        keyboard.pressed_keys[key as usize] = true;
        keyboard.most_recent_key = Some(key);

        keyboard.release_key(key);

        assert!(!keyboard.pressed_keys[key as usize]);
        assert_eq!(keyboard.most_recent_key, None);
    }
}