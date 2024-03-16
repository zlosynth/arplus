use crate::arpeggiator::Mode as ArpMode;
use crate::chords::Chord;
use crate::scales::GroupId as ScaleGroupId;

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
    pub prioritized: [Option<Page>; 5],
}

#[derive(Debug, defmt::Format)]
pub struct Page {
    clock: usize,
    screen: Screen,
}

#[derive(Debug, defmt::Format)]
pub enum Screen {
    Step(StepScreen),
    ArpMode(ArpModeScreen),
    Scale(ScaleScreen),
    ScaleGroup(ScaleGroupScreen),
    ChordGroup(ChordGroupScreen),
    Note(NoteScreen),
    Chord(ChordScreen),
}

#[derive(Debug, defmt::Format)]
pub struct StepScreen {
    step: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ArpModeScreen {
    mode: ArpMode,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleScreen {
    scale: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ScaleGroupScreen {
    scale_group: ScaleGroupId,
}

#[derive(Debug, defmt::Format)]
pub struct ChordGroupScreen {
    chord_group_size: usize,
}

#[derive(Debug, defmt::Format)]
pub struct NoteScreen {
    index: usize,
}

#[derive(Debug, defmt::Format)]
pub struct ChordScreen {
    chord: Chord,
}

impl Display {
    pub fn new() -> Self {
        Self {
            prioritized: [None, None, None, None, None],
        }
    }

    pub fn set(&mut self, priority: Priority, screen: Screen) {
        self.prioritized[priority as usize] = Some(Page::with_screen(screen));
    }

    pub fn reset(&mut self, priority: Priority) {
        self.prioritized[priority as usize] = None;
    }

    pub fn tick(&mut self) {
        for screen in self.prioritized.iter_mut().filter(|p| p.is_some()) {
            // SAFETY: The iterator already filters for `Some`.
            *screen = screen.take().unwrap().ticked();
        }
    }

    pub fn active_screen(&self) -> Option<&Screen> {
        self.prioritized
            .iter()
            .find_map(Option::as_ref)
            .map(|p| &p.screen)
    }
}

impl Page {
    fn with_screen(screen: Screen) -> Self {
        Self { clock: 0, screen }
    }

    fn ticked(mut self) -> Option<Self> {
        self.clock += 1;
        if self.clock > 2000 {
            None
        } else {
            Some(self)
        }
    }
}

impl Screen {
    pub fn arp_mode(arp_mode: ArpMode) -> Self {
        Screen::ArpMode(ArpModeScreen::with_selected(arp_mode))
    }

    pub fn scale_group(scale_group: ScaleGroupId) -> Self {
        Screen::ScaleGroup(ScaleGroupScreen::with_selected(scale_group))
    }

    pub fn scale(scale_index: usize) -> Self {
        Screen::Scale(ScaleScreen::with_index(scale_index))
    }

    pub fn chord_group(size: usize) -> Self {
        Screen::ChordGroup(ChordGroupScreen::with_size(size))
    }

    pub fn note(note_index: usize) -> Self {
        Screen::Note(NoteScreen::with_index(note_index))
    }

    pub fn chord(chord: Chord) -> Self {
        Screen::Chord(ChordScreen::with_selected(chord))
    }

    pub fn leds(&self) -> [bool; 8] {
        match self {
            Screen::Step(s) => s.leds(),
            Screen::ArpMode(s) => s.leds(),
            Screen::Scale(s) => s.leds(),
            Screen::ScaleGroup(s) => s.leds(),
            Screen::ChordGroup(s) => s.leds(),
            Screen::Note(s) => s.leds(),
            Screen::Chord(s) => s.leds(),
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
}

impl ArpModeScreen {
    pub fn with_selected(mode: ArpMode) -> Self {
        Self { mode }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(2 + self.mode as usize) {
            *led = true;
        }
        leds
    }
}

impl ScaleScreen {
    pub fn with_index(scale: usize) -> Self {
        Self { scale }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.scale) {
            *led = true;
        }
        leds
    }
}

impl ScaleGroupScreen {
    pub fn with_selected(scale_group: ScaleGroupId) -> Self {
        Self { scale_group }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.scale_group as usize) {
            *led = true;
        }
        leds
    }
}

impl ChordGroupScreen {
    pub fn with_size(chord_group_size: usize) -> Self {
        Self { chord_group_size }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.chord_group_size) {
            *led = true;
        }
        leds
    }
}

impl NoteScreen {
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8
        let mut leds = [false; 8];
        if let Some(led) = leds.get_mut(self.index) {
            *led = true;
        }
        leds
    }
}

impl ChordScreen {
    pub fn with_selected(chord: Chord) -> Self {
        Self { chord }
    }

    fn leds(&self) -> [bool; 8] {
        // TODO: Show properly steps above 8, if possible with chords.
        // TODO: Wrap around based on the selected scale and number of its steps.
        let mut leds = [false; 8];

        for step in self.chord.iter() {
            if let Some(led) = leds.get_mut(*step as usize) {
                *led = true;
            }
        }

        leds
    }
}
