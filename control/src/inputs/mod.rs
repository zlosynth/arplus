mod buffer;
mod button;
mod cv;
mod gate;
mod pot;

pub use button::Button;
pub use cv::{Cv, PersistentConfig as CvPersistentConfig};
pub use gate::Gate;
pub use pot::Pot;

pub struct ControlInputSnapshot {
    pub pots: [f32; 10],
    pub buttons: [bool; 7],
    pub cvs: [Option<f32>; 6],
    pub gates: [bool; 2],
}

pub struct Inputs {
    pub pots: Pots,
    pub cvs: Cvs,
    pub gates: Gates,
    pub buttons: Buttons,
}

pub struct Pots {
    pub tone: Pot,
    pub tonic: Pot,
    pub chord: Pot,
    pub chord_size: Pot,
    pub contour: Pot,
    pub cutoff: Pot,
    pub resonance: Pot,
    pub pluck: Pot,
    pub strings: Pot,
    pub width: Pot,
}

pub struct Cvs {
    pub tone: Cv,
    pub chord: Cv,
    pub assignable: Cv,
    pub contour: Cv,
    pub cutoff: Cv,
    pub resonance: Cv,
}

pub struct Gates {
    pub rsnx: Gate,
    pub trigger: Gate,
}

pub struct Buttons {
    pub group: Button,
    pub scale: Button,
    pub arp: Button,
    pub trigger: Button,
    pub rsnx: Button,
    pub stereo: Button,
    pub cv_assignment: Button,
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
                tonic: Pot::new(),
                chord: Pot::new(),
                chord_size: Pot::new(),
                contour: Pot::new(),
                cutoff: Pot::new(),
                resonance: Pot::new(),
                pluck: Pot::new(),
                strings: Pot::new(),
                width: Pot::new(),
            },
            cvs: Cvs {
                tone: Cv::with_config(config.tone_cv_calibration),
                chord: Cv::new(),
                contour: Cv::new(),
                cutoff: Cv::new(),
                resonance: Cv::new(),
                assignable: Cv::new(),
            },
            gates: Gates {
                rsnx: Gate::new(),
                trigger: Gate::new(),
            },
            buttons: Buttons {
                group: Button::new(),
                scale: Button::new(),
                arp: Button::new(),
                trigger: Button::new(),
                rsnx: Button::new(),
                stereo: Button::new(),
                cv_assignment: Button::new(),
            },
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) {
        self.pots.tone.reconcile(snapshot.pots[0]);
        self.pots.chord.reconcile(snapshot.pots[1]);
        self.pots.tonic.reconcile(snapshot.pots[2]);
        self.pots.chord_size.reconcile(snapshot.pots[3]);
        self.pots.resonance.reconcile(snapshot.pots[4]);
        self.pots.cutoff.reconcile(snapshot.pots[5]);
        self.pots.contour.reconcile(snapshot.pots[6]);
        self.pots.pluck.reconcile(snapshot.pots[7]);
        self.pots.strings.reconcile(snapshot.pots[8]);
        self.pots.width.reconcile(snapshot.pots[9]);

        self.cvs.tone.reconcile(snapshot.cvs[0]);
        self.cvs.chord.reconcile(snapshot.cvs[1]);
        self.cvs.assignable.reconcile(snapshot.cvs[2]);
        self.cvs.cutoff.reconcile(snapshot.cvs[4]);
        self.cvs.contour.reconcile(snapshot.cvs[3]);
        self.cvs.resonance.reconcile(snapshot.cvs[5]);

        self.gates.rsnx.reconcile(snapshot.gates[0]);
        self.gates.trigger.reconcile(snapshot.gates[1]);

        self.buttons.trigger.reconcile(snapshot.buttons[0]);
        self.buttons.rsnx.reconcile(snapshot.buttons[1]);
        self.buttons.arp.reconcile(snapshot.buttons[2]);
        self.buttons.group.reconcile(snapshot.buttons[3]);
        self.buttons.stereo.reconcile(snapshot.buttons[4]);
        self.buttons.cv_assignment.reconcile(snapshot.buttons[5]);
        self.buttons.scale.reconcile(snapshot.buttons[6]);
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            tone_cv_calibration: self.cvs.tone.copy_config(),
        }
    }
}
