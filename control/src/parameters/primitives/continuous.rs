pub struct Continuous {
    value: f32,
}

impl Continuous {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn reconcile(&mut self, value: f32) {
        self.value = value;
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}
