pub struct Keyboard {
    pub pressed_key: Option<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self { pressed_key: None }
    }
}
