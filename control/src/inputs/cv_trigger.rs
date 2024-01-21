pub struct CvTrigger {
    pub(crate) active: bool,
    pub triggered: bool,
}

impl CvTrigger {
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
}
