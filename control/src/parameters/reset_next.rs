pub struct ResetNext {
    high: bool,
}

impl ResetNext {
    pub fn new() -> Self {
        Self { high: false }
    }

    pub fn pop(&mut self) -> bool {
        let value = self.high;
        self.high = false;
        value
    }

    pub fn reconcile(&mut self, button_pushed: bool, cv_triggered: bool) {
        self.high |= button_pushed || cv_triggered;
    }
}
