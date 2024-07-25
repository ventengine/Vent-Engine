use vent_window::keyboard::{Key, KeyState};

#[derive(Default)]
pub struct InputHandler {
    pressed_keys: Vec<Key>,
}

impl InputHandler {
    pub fn set_key(&mut self, key: Key, state: KeyState) {
        if state == KeyState::Pressed {
            self.press_key(key)
        } else if state == KeyState::Released {
            self.release_key(key)
        }
    }

    pub fn press_key(&mut self, key: Key) {
        self.pressed_keys.push(key);
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn release_key(&mut self, key: Key) {
        let index = self.pressed_keys.iter().position(|x| *x == key);
        // We check because there might be invalid keys
        if let Some(index) = index {
            self.pressed_keys.remove(index);
        }
    }
}
