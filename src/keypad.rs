pub struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keys: [false; 16],
        }
    }

    pub fn is_pressed(&self, index: usize) -> bool {
        self.keys[index]
    }

    pub fn press(&mut self, index: usize) {
        self.keys[index] = true;
    }

    pub fn release(&mut self, index: usize) {
        self.keys[index] = false;
    }

    pub fn which_pressed(&self) -> Option<usize> {
        for (i, &key) in self.keys.iter().enumerate() {
            if key {
                return Some(i);
            }
        }
        None
    }
}