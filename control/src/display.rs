use crate::arpeggiator::Mode as ArpMode;

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
    ChordGroup(ChordGroupScreen),
    Note(NoteScreen),
}

impl Screen {
    pub fn arp_mode(mode: ArpMode) -> Self {
        Self::ArpMode(ArpModeScreen::with_selected(mode))
    }

    pub fn scale(scale: usize) -> Self {
        Self::Scale(ScaleScreen::with_selected(scale))
    }

    pub fn scale_group(scale_group: crate::scales::GroupId) -> Self {
        Self::ScaleGroup(ScaleGroupScreen::with_selected(scale_group))
    }

    pub fn chord_group(size: usize) -> Screen {
        Self::ChordGroup(ChordGroupScreen::with_size(size))
    }

    pub fn note(index: usize) -> Screen {
        Self::Note(NoteScreen::with_index(index))
    }
}

#[derive(Debug, defmt::Format)]
pub struct StepScreen {
    step: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ArpModeScreen {
    mode: ArpMode,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleScreen {
    scale: usize,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleGroupScreen {
    scale_group: crate::scales::GroupId,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ChordGroupScreen {
    chord_group_size: usize,
    countdown: usize,
}

#[derive(Debug, defmt::Format)]
pub struct NoteScreen {
    index: usize,
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
            Screen::ChordGroup(s) => s.leds(),
            Screen::Note(s) => s.leds(),
        }
    }

    pub fn ticked(self) -> Option<Self> {
        match self {
            Screen::Step(s) => s.ticked(),
            Screen::ArpMode(s) => s.ticked(),
            Screen::Scale(s) => s.ticked(),
            Screen::ScaleGroup(s) => s.ticked(),
            Screen::ChordGroup(s) => s.ticked(),
            Screen::Note(s) => s.ticked(),
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
    pub fn with_selected(mode: ArpMode) -> Self {
        Self {
            mode,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(2 + self.mode as usize) {
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
    pub fn with_selected(scale_group: crate::scales::GroupId) -> Self {
        Self {
            scale_group,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.scale_group as usize) {
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

impl ChordGroupScreen {
    pub fn with_size(chord_group_size: usize) -> Self {
        Self {
            chord_group_size,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.chord_group_size) {
            *led = true;
        }
        leds
    }

    fn ticked(mut self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        self.countdown -= 1;
        if self.countdown > 0 {
            Some(Screen::ChordGroup(self))
        } else {
            None
        }
    }
}

impl NoteScreen {
    pub fn with_index(index: usize) -> Self {
        Self {
            index,
            countdown: 2000,
        }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.index) {
            *led = true;
        }
        leds
    }

    fn ticked(mut self) -> Option<Screen> {
        // TODO: It's odd to return it wrapped in an outter type?
        self.countdown -= 1;
        if self.countdown > 0 {
            Some(Screen::Note(self))
        } else {
            None
        }
    }
}
