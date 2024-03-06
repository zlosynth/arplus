pub struct Trigger {
    triggered: bool,
}

impl Trigger {
    pub fn new() -> Self {
        Self { triggered: false }
    }

    pub fn triggered(&self) -> bool {
        self.triggered
    }

    pub fn reconcile(&mut self, button_pushed: bool, cv_triggered: bool) {
        self.triggered = button_pushed || cv_triggered;
    }
}
