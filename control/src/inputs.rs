pub struct ControlInputSnapshot {
    pub pots: [f32; 6],
    pub buttons: [bool; 4],
    pub cvs: [Option<f32>; 6],
    pub gates: [bool; 1],
}

pub struct Inputs {
    pots: Pots,
    cvs: Cvs,
    buttons: Buttons,
}

impl Inputs {
    pub fn new() -> Inputs {
        Self {
            pots: Pots {
                tone: Pot::new(),
                chord: Pot::new(),
                contour: Pot::new(),
                gain: Pot::new(),
                cutoff: Pot::new(),
                resonance: Pot::new(),
            },
            cvs: Cvs {
                tone: Cv::new(),
                chord: Cv::new(),
                contour: Cv::new(),
                gain: Cv::new(),
                cutoff: Cv::new(),
                resonance: Cv::new(),
                trigger: CvTrigger::new(),
            },
            buttons: Buttons {
                tonic: Button::new(),
                mode: Button::new(),
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
        self.pots.gain.reconcile(snapshot.pots[4]);
        self.pots.resonance.reconcile(snapshot.pots[5]);

        self.cvs.tone.reconcile(snapshot.cvs[0]);
        self.cvs.contour.reconcile(snapshot.cvs[1]);
        self.cvs.cutoff.reconcile(snapshot.cvs[2]);
        self.cvs.chord.reconcile(snapshot.cvs[3]);
        self.cvs.gain.reconcile(snapshot.cvs[4]);
        self.cvs.resonance.reconcile(snapshot.cvs[5]);
        self.cvs.trigger.reconcile(snapshot.gates[0]);

        self.buttons.tonic.reconcile(snapshot.buttons[0]);
        self.buttons.mode.reconcile(snapshot.buttons[1]);
        self.buttons.arp.reconcile(snapshot.buttons[2]);
        self.buttons.trigger.reconcile(snapshot.buttons[3]);
    }
}

pub struct Pots {
    pub tone: Pot,
    pub chord: Pot,
    pub contour: Pot,
    pub gain: Pot,
    pub cutoff: Pot,
    pub resonance: Pot,
}

pub struct Cvs {
    pub tone: Cv,
    pub chord: Cv,
    pub contour: Cv,
    pub gain: Cv,
    pub cutoff: Cv,
    pub resonance: Cv,
    pub trigger: CvTrigger,
}

pub struct Buttons {
    pub tonic: Button,
    pub mode: Button,
    pub arp: Button,
    pub trigger: Button,
}

pub struct Pot {
    pub value: f32,
}

impl Pot {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn reconcile(&mut self, value: f32) {
        self.value = value;
    }
}

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

pub struct CvTrigger {
    active: bool,
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

pub struct Button {
    pub pressed: bool,
    pub clicked: bool,
    pub released: bool,
    pub held_for: usize,
}

impl Button {
    pub fn new() -> Self {
        Self {
            pressed: false,
            clicked: false,
            released: false,
            held_for: 0,
        }
    }

    pub fn reconcile(&mut self, pressed: bool) {
        self.clicked = !self.pressed && pressed;
        self.released = self.pressed && !pressed;
        self.pressed = pressed;
        if pressed {
            self.held_for += 1;
        } else {
            self.held_for = 0;
        }
    }
}
