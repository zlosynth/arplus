use daisy::led::LedUser;
use fugit::{Duration, ExtU64};
use stm32h7xx_hal::gpio::PinState;

pub struct VersionIndicator {
    blinks: u8,
    led: LedUser,
    phase: Phase,
}

enum Phase {
    LongOff,
    ShortOff(u8),
    On(u8),
}

impl VersionIndicator {
    pub fn new(blinks: u8, led: LedUser) -> Self {
        Self {
            blinks,
            led,
            phase: Phase::LongOff,
        }
    }

    pub fn cycle(&mut self) -> Duration<u64, 1, 1000> {
        self.phase = match self.phase {
            Phase::LongOff => Phase::On(0),
            Phase::ShortOff(i) => Phase::On(i + 1),
            Phase::On(i) => {
                let finished_blinks = i == self.blinks - 1;
                if finished_blinks {
                    Phase::LongOff
                } else {
                    Phase::ShortOff(i)
                }
            }
        };
        self.led.set_state(self.phase.required_led_state());
        self.phase.required_sleep()
    }
}

impl Phase {
    fn required_sleep(&self) -> Duration<u64, 1, 1000> {
        match self {
            Phase::LongOff => 2.secs(),
            Phase::ShortOff(_) | Phase::On(_) => 200.millis(),
        }
    }

    fn required_led_state(&self) -> PinState {
        match self {
            Phase::LongOff => PinState::Low,
            Phase::ShortOff(_) | Phase::On(_) => PinState::High,
        }
    }
}
