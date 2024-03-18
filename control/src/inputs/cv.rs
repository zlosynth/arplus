use super::calibration::Calibration;

pub struct Cv {
    just_plugged: bool,
    value: Option<f32>,
    calibration: Option<Calibration>,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub calibration: Calibration,
}

impl Cv {
    pub fn new() -> Self {
        Self {
            just_plugged: false,
            value: None,
            calibration: None,
        }
    }

    pub fn with_config(config: PersistentConfig) -> Self {
        Self {
            just_plugged: false,
            value: None,
            calibration: Some(config.calibration),
        }
    }

    pub fn update_calibration(&mut self, octave_1: f32, octave_2: f32) -> Result<(), ()> {
        let new_calibration = Calibration::try_new(octave_1, octave_2)?;
        self.calibration = Some(new_calibration);
        Ok(())
    }

    pub fn reconcile(&mut self, value: Option<f32>) {
        self.just_plugged = self.value.is_none() && value.is_some();

        self.value = if let Some(calibration) = self.calibration {
            value.map(|x| calibration.apply(x))
        } else {
            value
        };
    }

    pub fn value(&self) -> Option<f32> {
        self.value
    }

    pub fn just_plugged(&self) -> bool {
        self.just_plugged
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            calibration: self.calibration.unwrap(),
        }
    }
}
