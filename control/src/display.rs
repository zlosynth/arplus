use crate::arpeggiator::Mode as ArpMode;
use crate::chords::Chord;
use crate::parameters::CvMappingSocket;
use crate::scales::{GroupId, Tonic};

#[repr(u8)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum Priority {
    Failure = 0,
    Dialog,
    Queried,
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

#[derive(Debug, defmt::Format, PartialEq)]
pub enum Screen {
    Failure(FailureScreen),
    Step(StepScreen),
    ArpMode(ArpModeScreen),
    Scale(ScaleScreen),
    Group(GroupScreen),
    Size(SizeScreen),
    Note(NoteScreen),
    Chord(ChordScreen),
    ToneCalibration(ToneCalibrationScreen),
    Octave(OctaveScreen),
    Tonic(TonicScreen),
    Configuration(ConfigurationScreen),
    Gain(GainScreen),
    CvMapping(CvMappingScreen),
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct StepScreen {
    step: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct ArpModeScreen {
    mode: ArpMode,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct ScaleScreen {
    scale: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct GroupScreen {
    group: GroupId,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct SizeScreen {
    size: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct NoteScreen {
    index: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct ChordScreen {
    chord: Chord,
    scale_size: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct OctaveScreen {
    index: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct TonicScreen {
    tonic: Tonic,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct GainScreen {
    index: usize,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct CvMappingScreen {
    socket: CvMappingSocket,
}

#[derive(Debug, defmt::Format, PartialEq)]
pub struct ConfigurationScreen;

#[derive(Debug, defmt::Format, PartialEq)]
pub struct FailureScreen;

#[derive(Debug, defmt::Format, PartialEq)]
pub enum ToneCalibrationScreen {
    Octave1,
    Octave2,
    Failure,
    Success,
}

impl Display {
    pub fn new() -> Self {
        Self {
            prioritized: [None, None, None, None, None],
        }
    }

    pub fn set(&mut self, priority: Priority, screen: Screen) {
        if let Some(page) = self.prioritized[priority as usize].as_ref() {
            if page.screen == screen {
                return;
            }
        }

        self.prioritized[priority as usize] = Some(Page::with_screen(screen));
    }

    pub fn reset(&mut self, priority: Priority) {
        self.prioritized[priority as usize] = None;
    }

    pub fn tick(&mut self) {
        for page in self.prioritized.iter_mut().flatten() {
            page.tick();
        }

        if let Some(page) = self.prioritized[Priority::Failure as usize].as_ref() {
            if page.clock > 2000 {
                self.reset(Priority::Failure);
            }
        }

        if let Some(page) = self.prioritized[Priority::Queried as usize].as_ref() {
            if page.clock > 2000 {
                self.reset(Priority::Queried);
            }
        }
    }

    pub fn active_screen_and_clock(&self) -> Option<(&Screen, usize)> {
        self.prioritized
            .iter()
            .find_map(Option::as_ref)
            .map(|p| (&p.screen, p.clock))
    }
}

impl Page {
    fn with_screen(screen: Screen) -> Self {
        Self { clock: 0, screen }
    }

    fn tick(&mut self) {
        self.clock = self.clock.wrapping_add(1);
    }
}

impl Screen {
    pub fn arp_mode(arp_mode: ArpMode) -> Self {
        Screen::ArpMode(ArpModeScreen::with_selected(arp_mode))
    }

    pub fn group(group: GroupId) -> Self {
        Screen::Group(GroupScreen::with_selected(group))
    }

    pub fn scale(scale_index: usize) -> Self {
        Screen::Scale(ScaleScreen::with_index(scale_index))
    }

    pub fn size(size: usize) -> Self {
        Screen::Size(SizeScreen::with_size(size))
    }

    pub fn note(note_index: usize) -> Self {
        Screen::Note(NoteScreen::with_index(note_index))
    }

    pub fn step(step_index: usize) -> Self {
        Screen::Step(StepScreen::with_step(step_index))
    }

    pub fn chord(chord: Chord, scale_size: usize) -> Self {
        Screen::Chord(ChordScreen::with_selected(chord, scale_size))
    }

    pub fn octave(index: usize) -> Self {
        Screen::Octave(OctaveScreen::with_index(index))
    }

    pub fn tonic(tonic: Tonic) -> Self {
        Screen::Tonic(TonicScreen::with_tonic(tonic))
    }

    pub fn configuration() -> Self {
        Screen::Configuration(ConfigurationScreen)
    }

    pub fn gain(index: usize) -> Screen {
        Screen::Gain(GainScreen::with_index(index))
    }

    pub fn cv_mapping(socket: CvMappingSocket) -> Screen {
        Screen::CvMapping(CvMappingScreen::with_socket(socket))
    }

    pub fn failure() -> Self {
        Screen::Failure(FailureScreen)
    }

    pub fn tone_calibration_octave_1() -> Self {
        Screen::ToneCalibration(ToneCalibrationScreen::Octave1)
    }

    pub fn tone_calibration_octave_2() -> Self {
        Screen::ToneCalibration(ToneCalibrationScreen::Octave2)
    }

    pub fn calibration_success() -> Self {
        Screen::ToneCalibration(ToneCalibrationScreen::Success)
    }

    pub fn calibration_failure() -> Self {
        Screen::ToneCalibration(ToneCalibrationScreen::Failure)
    }

    pub fn leds(&self, clock: usize) -> [bool; 8] {
        match self {
            Screen::Step(s) => s.leds(),
            Screen::ArpMode(s) => s.leds(),
            Screen::Scale(s) => s.leds(),
            Screen::Group(s) => s.leds(),
            Screen::Size(s) => s.leds(),
            Screen::Note(s) => s.leds(),
            Screen::Chord(s) => s.leds(),
            Screen::Octave(s) => s.leds(),
            Screen::Tonic(s) => s.leds(),
            Screen::Gain(s) => s.leds(),
            Screen::CvMapping(s) => s.leds(),
            Screen::Failure(s) => s.leds(clock),
            Screen::ToneCalibration(s) => s.leds(clock),
            Screen::Configuration(s) => s.leds(clock),
        }
    }
}

impl StepScreen {
    pub fn with_step(step: usize) -> Self {
        Self { step }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        if self.step >= leds.len() {
            leds[leds.len() - 1] = true;
        }
        leds[self.step % leds.len()] = true;
        leds
    }
}

impl ArpModeScreen {
    pub fn with_selected(mode: ArpMode) -> Self {
        Self { mode }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        let leds_max = leds.len() - 1;
        if let Some(led) = leds.get_mut(leds_max - self.mode as usize) {
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
        let mut leds = [false; 8];
        let leds_max = leds.len() - 1;
        if let Some(led) = leds.get_mut(leds_max - self.scale) {
            *led = true;
        }
        leds
    }
}

impl GroupScreen {
    pub fn with_selected(group: GroupId) -> Self {
        Self { group }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        let leds_max = leds.len() - 1;
        if let Some(led) = leds.get_mut(leds_max - self.group as usize) {
            *led = true;
        }
        leds
    }
}

impl SizeScreen {
    pub fn with_size(size: usize) -> Self {
        Self { size }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        if self.size <= leds.len() {
            for led in leds[..self.size].iter_mut() {
                *led = true;
            }
        } else {
            for (i, led) in leds.iter_mut().enumerate() {
                if i % 2 == 0 {
                    *led = true;
                }
            }
        }
        leds
    }
}

impl NoteScreen {
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        if self.index >= leds.len() {
            leds[leds.len() - 1] = true;
        }
        leds[self.index % leds.len()] = true;
        leds
    }
}

impl ChordScreen {
    pub fn with_selected(chord: Chord, scale_size: usize) -> Self {
        Self { chord, scale_size }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];

        let wrap = usize::min(self.scale_size, leds.len());

        for step in self.chord.iter() {
            if let Some(led) = leds.get_mut(*step as usize % wrap) {
                *led = true;
            }
        }

        leds
    }
}

impl OctaveScreen {
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    fn leds(&self) -> [bool; 8] {
        match self.index {
            0 => [true, true, true, true, false, false, false, false],
            1 => [false, false, false, false, true, true, true, true],
            _ => [true; 8],
        }
    }
}

impl TonicScreen {
    pub fn with_tonic(tonic: Tonic) -> Self {
        Self { tonic }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        match self.tonic {
            Tonic::C => leds[0] = true,
            Tonic::CSharp => {
                leds[0] = true;
                leds[7] = true;
            }
            Tonic::D => leds[1] = true,
            Tonic::DSharp => {
                leds[1] = true;
                leds[7] = true;
            }
            Tonic::E => leds[2] = true,
            Tonic::F => leds[3] = true,
            Tonic::FSharp => {
                leds[3] = true;
                leds[7] = true;
            }
            Tonic::G => leds[4] = true,
            Tonic::GSharp => {
                leds[4] = true;
                leds[7] = true;
            }
            Tonic::A => leds[5] = true,
            Tonic::ASharp => {
                leds[5] = true;
                leds[7] = true;
            }
            Tonic::B => leds[6] = true,
        }
        leds
    }
}

impl GainScreen {
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];
        let len = leds.len();
        for led in leds[0..usize::min((self.index + 1) * 2, len)].iter_mut() {
            *led = true;
        }
        leds
    }
}

impl CvMappingScreen {
    pub fn with_socket(socket: CvMappingSocket) -> Self {
        Self { socket }
    }

    fn leds(&self) -> [bool; 8] {
        let mut leds = [false; 8];

        if self.socket.is_none() {
            return leds;
        }

        let index = self.socket as usize - 1;
        if let Some(led) = leds.get_mut(index) {
            *led = true;
        }

        leds
    }
}

impl FailureScreen {
    fn leds(&self, clock: usize) -> [bool; 8] {
        let phase = (clock / 400) % 2;
        if phase == 0 {
            [true; 8]
        } else {
            [false; 8]
        }
    }
}

impl ConfigurationScreen {
    fn leds(&self, clock: usize) -> [bool; 8] {
        let phase = (clock / 400) % 2;
        if phase == 0 {
            [true, false, true, false, true, false, true, false]
        } else {
            [false, true, false, true, false, true, false, true]
        }
    }
}

impl ToneCalibrationScreen {
    fn leds(&self, clock: usize) -> [bool; 8] {
        match self {
            ToneCalibrationScreen::Octave1 => {
                let phase = (clock / 400) % 6;
                if phase == 0 || phase == 2 {
                    [true, true, true, true, false, false, false, false]
                } else {
                    [false, false, false, false, false, false, false, false]
                }
            }
            ToneCalibrationScreen::Octave2 => {
                let phase = (clock / 400) % 6;
                if phase == 0 || phase == 2 {
                    [false, false, false, false, true, true, true, true]
                } else {
                    [false, false, false, false, false, false, false, false]
                }
            }
            ToneCalibrationScreen::Failure => {
                [false, false, false, false, false, false, false, false]
            }
            ToneCalibrationScreen::Success => [true, true, true, true, true, true, true, true],
        }
    }
}
