pub struct Cv {
    pub value: Option<f32>,
}

impl Cv {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn reconcile(&mut self, value: Option<f32>) {
        self.value = value;
    }
}
