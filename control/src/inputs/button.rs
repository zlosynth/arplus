pub struct Button {
    pressed: bool,
    clicked: bool,
    released: bool,
    held_for: usize,
    released_after: usize,
}

impl Button {
    pub fn new() -> Self {
        Self {
            pressed: false,
            clicked: false,
            released: false,
            held_for: 0,
            released_after: 0,
        }
    }

    pub fn reconcile(&mut self, pressed: bool) {
        self.clicked = !self.pressed && pressed;
        self.released = self.pressed && !pressed;
        if self.released {
            self.released_after = self.held_for;
        }
        self.pressed = pressed;
        if pressed {
            self.held_for += 1;
        } else {
            self.held_for = 0;
        }
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn released(&self) -> bool {
        self.released
    }

    pub fn held_for(&self) -> usize {
        self.held_for
    }

    pub fn released_after(&self) -> usize {
        self.released_after
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }
}
