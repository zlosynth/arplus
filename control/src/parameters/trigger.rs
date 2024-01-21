pub struct DualTrigger {
    triggered: bool,
}

impl DualTrigger {
    pub fn new() -> Self {
        Self { triggered: false }
    }

    pub fn triggered(&self) -> bool {
        self.triggered
    }

    pub fn reconcile(&mut self, triggered: bool) {
        self.triggered = triggered;
    }
}
