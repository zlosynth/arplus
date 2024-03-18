mod button;
mod calibration;
mod cv;
mod gate;
mod pot;

pub use button::Button;
pub use cv::{Cv, PersistentConfig as CvPersistentConfig};
pub use gate::Gate;
pub use pot::Pot;

pub struct ControlInputSnapshot {
    pub pots: [f32; 6],
    pub buttons: [bool; 4],
    pub cvs: [Option<f32>; 6],
    pub gates: [bool; 1],
}

pub struct Inputs {
    pub pots: Pots,
    pub cvs: Cvs,
    pub gates: Gates,
    pub buttons: Buttons,
}

pub struct Pots {
    pub tone: Pot,
    pub chord: Pot,
    pub chord_group: Pot,
    pub contour: Pot,
    pub cutoff: Pot,
    pub resonance: Pot,
}

pub struct Cvs {
    pub tone: Cv,
    pub chord: Cv,
    pub chord_group: Cv,
    pub contour: Cv,
    pub cutoff: Cv,
    pub resonance: Cv,
}

pub struct Gates {
    pub trigger: Gate,
}

pub struct Buttons {
    pub scale_group: Button,
    pub scale: Button,
    pub arp: Button,
    pub trigger: Button,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    pub tone_cv_calibration: CvPersistentConfig,
}

impl Inputs {
    pub fn new(config: PersistentConfig) -> Inputs {
        Self {
            pots: Pots {
                tone: Pot::new(),
                chord: Pot::new(),
                chord_group: Pot::new(),
                contour: Pot::new(),
                cutoff: Pot::new(),
                resonance: Pot::new(),
            },
            cvs: Cvs {
                tone: Cv::with_config(config.tone_cv_calibration),
                chord: Cv::new(),
                chord_group: Cv::new(),
                contour: Cv::new(),
                cutoff: Cv::new(),
                resonance: Cv::new(),
            },
            gates: Gates {
                trigger: Gate::new(),
            },
            buttons: Buttons {
                scale_group: Button::new(),
                scale: Button::new(),
                arp: Button::new(),
                trigger: Button::new(),
            },
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) {
        self.pots.tone.reconcile(snapshot.pots[0]);
        self.pots.contour.reconcile(snapshot.pots[1]);
        self.pots.cutoff.reconcile(snapshot.pots[2]);
        self.pots.chord.reconcile(snapshot.pots[3]);
        self.pots.chord_group.reconcile(snapshot.pots[4]);
        self.pots.resonance.reconcile(snapshot.pots[5]);

        self.cvs.tone.reconcile(snapshot.cvs[0]);
        self.cvs.contour.reconcile(snapshot.cvs[1]);
        self.cvs.cutoff.reconcile(snapshot.cvs[2]);
        self.cvs.chord.reconcile(snapshot.cvs[3]);
        self.cvs.chord_group.reconcile(snapshot.cvs[4]);
        self.cvs.resonance.reconcile(snapshot.cvs[5]);

        self.gates.trigger.reconcile(snapshot.gates[0]);

        self.buttons.scale_group.reconcile(snapshot.buttons[0]);
        self.buttons.scale.reconcile(snapshot.buttons[1]);
        self.buttons.arp.reconcile(snapshot.buttons[2]);
        self.buttons.trigger.reconcile(snapshot.buttons[3]);
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone_cv_calibration: self.cvs.tone.copy_config(),
        }
    }
    // TODO: Save config
}
