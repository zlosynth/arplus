pub struct Trigger {
    triggered: bool,
    triggered_n1: bool,
}

impl Trigger {
    pub fn new() -> Self {
        Self {
            triggered: false,
            triggered_n1: false,
        }
    }

    pub fn triggered_n1(&self) -> bool {
        self.triggered_n1
    }

    pub fn reconcile(&mut self, button_pushed: bool, cv_triggered: bool) {
        self.triggered_n1 = self.triggered;
        self.triggered = button_pushed || cv_triggered;
    }
}
