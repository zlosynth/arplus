#[repr(u8)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum Priority {
    Failure = 0,
    Dialog,
    Active,
    Fallback,
}

pub struct Display {
    pub prioritized: [Option<Screen>; 5],
}

#[derive(Debug, defmt::Format)]
pub enum Screen {
    CurrentStep(CurrentStepScreen),
}

#[derive(Debug, defmt::Format)]
pub struct CurrentStepScreen {
    step: usize,
}

impl Display {
    pub fn new() -> Self {
        Self {
            prioritized: [None, None, None, None, None],
        }
    }

    pub fn set(&mut self, priority: Priority, screen: Screen) {
        self.prioritized[priority as usize] = Some(screen);
    }

    pub fn tick(&mut self) {
        // TODO: Safety
        for screen in self.prioritized.iter_mut().filter(|p| p.is_some()) {
            *screen = screen.take().unwrap().ticked();
        }
    }

    pub fn active_screen(&self) -> Option<&Screen> {
        self.prioritized.iter().find_map(Option::as_ref)
    }
}

impl Screen {
    pub fn leds(&self) -> [bool; 8] {
        match self {
            Screen::CurrentStep(s) => s.leds(),
        }
    }

    pub fn ticked(self) -> Option<Self> {
        match self {
            Screen::CurrentStep(s) => s.ticked(),
        }
    }
}

impl CurrentStepScreen {
    pub fn with_step(step: usize) -> CurrentStepScreen {
        Self { step }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.step) {
            *led = true;
        }
        leds
    }

    fn ticked(self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        Some(Screen::CurrentStep(self))
    }
}
