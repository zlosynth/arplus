pub struct Button {
    pub pressed: bool,
    pub clicked: bool,
    pub released: bool,
    pub held_for: usize,
}

impl Button {
    pub fn new() -> Self {
        Self {
            pressed: false,
            clicked: false,
            released: false,
            held_for: 0,
        }
    }

    pub fn reconcile(&mut self, pressed: bool) {
        self.clicked = !self.pressed && pressed;
        self.released = self.pressed && !pressed;
        self.pressed = pressed;
        if pressed {
            self.held_for += 1;
        } else {
            self.held_for = 0;
        }
    }
}
