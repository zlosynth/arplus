use crate::calibration::Calibration;

pub struct QuantizedOutput {
    value: f32,
    forced_value: Option<f32>,
    calibration: Option<Calibration>,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub calibration: Calibration,
}

impl QuantizedOutput {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            forced_value: None,
            calibration: None,
        }
    }

    pub fn with_config(config: PersistentConfig) -> Self {
        Self {
            value: 0.0,
            forced_value: None,
            calibration: Some(config.calibration),
        }
    }

    pub fn update_calibration(&mut self, octave_1: f32, octave_2: f32) -> Result<(), ()> {
        let new_calibration = Calibration::try_new(octave_1, octave_2)?;
        self.calibration = Some(new_calibration);
        Ok(())
    }

    pub fn reconcile(&mut self, value: f32) {
        self.value = if let Some(calibration) = self.calibration {
            calibration.apply(value)
        } else {
            value
        };
    }

    pub fn value(&self) -> f32 {
        self.forced_value.unwrap_or(self.value)
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            calibration: self.calibration.unwrap(),
        }
    }

    pub fn force_octave_1(&mut self) {
        self.forced_value = Some(2.0);
    }

    pub fn force_octave_2(&mut self) {
        self.forced_value = Some(3.0);
    }

    pub fn remove_force(&mut self) {
        self.forced_value = None;
    }
}
