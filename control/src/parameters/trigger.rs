pub struct Trigger {
    triggered: bool,
    triggered_n1: bool,
    triggered_n2: bool,
    triggered_n3: bool,
    triggered_n4: bool,
}

impl Trigger {
    pub fn new() -> Self {
        Self {
            triggered: false,
            triggered_n1: false,
            triggered_n2: false,
            triggered_n3: false,
            triggered_n4: false,
        }
    }

    pub fn triggered_n4(&self) -> bool {
        self.triggered_n4
    }

    pub fn reconcile(&mut self, button_pushed: bool, cv_triggered: bool) {
        self.triggered_n4 = self.triggered_n3;
        self.triggered_n3 = self.triggered_n2;
        self.triggered_n2 = self.triggered_n1;
        self.triggered_n1 = self.triggered;
        self.triggered = button_pushed || cv_triggered;
    }
}
