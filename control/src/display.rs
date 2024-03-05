#[repr(u8)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum Priority {
    Failure = 0,
    Dialog,
    Queried,
    Active,
    Fallback,
}

pub struct Display {
    pub prioritized: [Option<Screen>; 5],
}

#[derive(Debug, defmt::Format)]
pub enum Screen {
    Step(StepScreen),
    ArpMode(ArpModeScreen),
    Scale(ScaleScreen),
    ScaleGroup(ScaleGroupScreen),
}

impl Screen {
    pub fn arp_mode(mode: usize) -> Self {
        Self::ArpMode(ArpModeScreen::with_selected(mode))
    }

    pub fn scale(scale: usize) -> Self {
        Self::Scale(ScaleScreen::with_selected(scale))
    }

    pub fn scale_group(scale_group: usize) -> Self {
        Self::ScaleGroup(ScaleGroupScreen::with_selected(scale_group))
    }
}

#[derive(Debug, defmt::Format)]
pub struct StepScreen {
    step: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ArpModeScreen {
    mode: usize,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleScreen {
    scale: usize,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleGroupScreen {
    scale_group: usize,
    countdown: usize,
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

    pub fn reset(&mut self, priority: Priority) {
        self.prioritized[priority as usize] = None;
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
            Screen::Step(s) => s.leds(),
            Screen::ArpMode(s) => s.leds(),
            Screen::Scale(s) => s.leds(),
            Screen::ScaleGroup(s) => s.leds(),
        }
    }

    pub fn ticked(self) -> Option<Self> {
        match self {
            Screen::Step(s) => s.ticked(),
            Screen::ArpMode(s) => s.ticked(),
            Screen::Scale(s) => s.ticked(),
            Screen::ScaleGroup(s) => s.ticked(),
        }
    }
}

impl StepScreen {
    pub fn with_step(step: usize) -> Self {
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
        Some(Screen::Step(self))
    }
}

impl ArpModeScreen {
    pub fn with_selected(mode: usize) -> Self {
        Self {
            mode,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(2 + self.mode) {
            *led = true;
        }
        leds
    }

    fn ticked(mut self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        self.countdown -= 1;
        if self.countdown > 0 {
            Some(Screen::ArpMode(self))
        } else {
            None
        }
    }
}

impl ScaleScreen {
    pub fn with_selected(scale: usize) -> Self {
        Self {
            scale,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.scale) {
            *led = true;
        }
        leds
    }

    fn ticked(mut self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        self.countdown -= 1;
        if self.countdown > 0 {
            Some(Screen::Scale(self))
        } else {
            None
        }
    }
}

impl ScaleGroupScreen {
    pub fn with_selected(scale_group: usize) -> Self {
        Self {
            scale_group,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.scale_group) {
            *led = true;
        }
        leds
    }

    fn ticked(mut self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        self.countdown -= 1;
        if self.countdown > 0 {
            Some(Screen::ScaleGroup(self))
        } else {
            None
        }
    }
}
