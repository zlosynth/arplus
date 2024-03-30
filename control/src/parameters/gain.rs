use super::primitives::discrete::Discrete;
pub use super::primitives::discrete::PersistentConfig;

const LEVELS: usize = 4;
const MIN: f32 = 0.2;
const MAX: f32 = 0.6;

pub struct Gain {
    discrete: Discrete,
}

impl Gain {
    pub fn new(config: PersistentConfig) -> Self {
        let mut discrete = Discrete::new(config, LEVELS, 0.1);
        discrete.reconcile(1.0);
        Self { discrete }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        self.discrete.copy_config()
    }

    pub fn reconcile(&mut self, input_level: f32) -> bool {
        self.discrete.reconcile(input_level)
    }

    pub fn value(&self) -> f32 {
        let phase = self.discrete.selected_value() as f32 / (LEVELS - 1) as f32;
        MIN + (MAX - MIN) * phase
    }

    pub fn selected_index(&self) -> usize {
        self.discrete.selected_value()
    }
}
