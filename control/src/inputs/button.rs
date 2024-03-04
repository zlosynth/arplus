pub struct Button {
    pub pressed: bool,
    pub clicked: bool,
    pub released: bool,
    pub held_for: usize,
    pub released_after: usize,
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
}
