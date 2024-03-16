pub struct Gate {
    active: bool,
    triggered: bool,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            active: false,
            triggered: false,
        }
    }

    pub fn reconcile(&mut self, active: bool) {
        self.triggered = !self.active && active;
        self.active = active;
    }

    pub fn triggered(&self) -> bool {
        self.triggered
    }
}
