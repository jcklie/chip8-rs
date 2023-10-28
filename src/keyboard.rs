#[derive(PartialEq, Debug)]
enum WaitingState {
    Waiting,
    Pressed { key: u8 },
    None,
}

pub struct Keyboard {
    pressed_keys: [bool; 16],
    waiting_state: WaitingState,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: [false; 16],
            waiting_state: WaitingState::None,
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.pressed_keys[key as usize]
    }

    pub fn press_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = true;
    }

    pub fn release_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = false;

        if self.waiting_state == WaitingState::Waiting {
            self.waiting_state = WaitingState::Pressed { key };
        }
    }

    pub fn wait_for_keypress(&mut self) -> Option<u8> {
        if self.waiting_state == WaitingState::None {
            self.waiting_state = WaitingState::Waiting;
            None
        } else if let WaitingState::Pressed { key } = self.waiting_state {
            self.waiting_state = WaitingState::None;
            Some(key)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim;

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
    }

    #[test]
    fn test_release_key() {
        let mut keyboard = Keyboard::new();

        let key: u8 = 0xE;
        keyboard.pressed_keys[key as usize] = true;

        keyboard.release_key(key);

        assert!(!keyboard.pressed_keys[key as usize]);
    }

    #[test]
    fn test_waiting() {
        let key = 0x8;

        let mut keyboard = Keyboard::new();
        assert_eq!(keyboard.waiting_state, WaitingState::None);

        keyboard.press_key(key);
        assert_eq!(keyboard.waiting_state, WaitingState::None);
        keyboard.release_key(key);

        // Start waiting
        assert_eq!(keyboard.wait_for_keypress(), None);
        assert_eq!(keyboard.waiting_state, WaitingState::Waiting);

        // Still waiting
        assert_eq!(keyboard.wait_for_keypress(), None);
        assert_eq!(keyboard.waiting_state, WaitingState::Waiting);

        // Pressing but not released, thus still waiting
        keyboard.press_key(key);
        assert_eq!(keyboard.wait_for_keypress(), None);
        assert_eq!(keyboard.waiting_state, WaitingState::Waiting);

        // Releasing, stop waiting
        keyboard.release_key(key);
        assert_eq!(keyboard.waiting_state, WaitingState::Pressed { key });
        assert_eq!(keyboard.wait_for_keypress(), Some(key));

        // Stop waiting
        assert_eq!(keyboard.waiting_state, WaitingState::None);
    }
}
